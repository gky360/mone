//! Input bandwidth from libc getifaddr function.

use crate::reader::{InterfaceInfoItem, InterfaceStat, InterfaceStats, Read};
use crate::utils::NumBytes;
use crate::{Error, Result};
use libc::c_void;
use nix::{net::if_::InterfaceFlags, sys::socket::SockAddr};
use std::{ffi, ptr};

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct IfData {
    ifi_ibytes: u32,
    ifi_obytes: u32,
}

impl IfData {
    #[cfg(target_os = "linux")]
    unsafe fn from_ifa_data(ifa_data: *mut c_void) -> Option<IfData> {
        use rtnetlink::packet::LinkStatsBuffer;
        const LINK_STATS32_LEN: usize = 24 * 4;

        if ifa_data.is_null() {
            return None;
        }

        let data_bytes: &[u8; LINK_STATS32_LEN] = &*(ifa_data as *const [u8; LINK_STATS32_LEN]);
        let buf = LinkStatsBuffer::new(&data_bytes[..]);
        Some(IfData {
            ifi_ibytes: buf.rx_bytes(),
            ifi_obytes: buf.tx_bytes(),
        })
    }

    #[cfg(not(target_os = "linux"))]
    unsafe fn from_ifa_data(ifa_data: *mut c_void) -> Option<IfData> {
        use libc::if_data;

        if ifa_data.is_null() {
            return None;
        }

        let data: if_data = *(ifa_data as *const if_data);
        Some(IfData {
            ifi_ibytes: data.ifi_ibytes,
            ifi_obytes: data.ifi_obytes,
        })
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct InterfaceAddress {
    /// Name of the network interface
    pub interface_name: String,
    /// Flags as from `SIOCGIFFLAGS` ioctl
    pub flags: InterfaceFlags,
    /// Network address of this interface
    pub address: Option<SockAddr>,
    /// Netmask of this interface
    pub netmask: Option<SockAddr>,
    /// Broadcast address of this interface, if applicable
    pub broadcast: Option<SockAddr>,
    /// Point-to-point destination address
    pub destination: Option<SockAddr>,
    /// address-family-specific data
    pub data: Option<IfData>,
}

cfg_if! {
    if #[cfg(any(target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))] {
        fn get_ifu_from_sockaddr(info: &libc::ifaddrs) -> *const libc::sockaddr {
            info.ifa_ifu
        }
    } else {
        fn get_ifu_from_sockaddr(info: &libc::ifaddrs) -> *const libc::sockaddr {
            info.ifa_dstaddr
        }
    }
}

impl InterfaceAddress {
    /// Create an `InterfaceAddress` from the libc struct.
    fn from_libc_ifaddrs(info: &libc::ifaddrs) -> InterfaceAddress {
        let ifname = unsafe { ffi::CStr::from_ptr(info.ifa_name) };
        let address = unsafe { SockAddr::from_libc_sockaddr(info.ifa_addr) };
        let netmask = unsafe { SockAddr::from_libc_sockaddr(info.ifa_netmask) };
        let data = unsafe { IfData::from_ifa_data(info.ifa_data) };

        let mut addr = InterfaceAddress {
            interface_name: ifname.to_string_lossy().to_string(),
            flags: InterfaceFlags::from_bits_truncate(info.ifa_flags as i32),
            address,
            netmask,
            broadcast: None,
            destination: None,
            data,
        };

        let ifu = get_ifu_from_sockaddr(info);
        if addr.flags.contains(InterfaceFlags::IFF_POINTOPOINT) {
            addr.destination = unsafe { SockAddr::from_libc_sockaddr(ifu) };
        } else if addr.flags.contains(InterfaceFlags::IFF_BROADCAST) {
            addr.broadcast = unsafe { SockAddr::from_libc_sockaddr(ifu) };
        }

        addr
    }
}

/// Holds the results of `getifaddrs`.
///
/// Use the function `getifaddrs` to create this Iterator. Note that the
/// actual list of interfaces can be iterated once and will be freed as
/// soon as the Iterator goes out of scope.
#[derive(Debug, Eq, Hash, PartialEq)]
struct InterfaceAddressIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

fn getifaddrs() -> Result<InterfaceAddressIterator> {
    let mut addrs: *mut libc::ifaddrs = ptr::null_mut();
    match nix::errno::Errno::result(unsafe { libc::getifaddrs(&mut addrs) }) {
        Ok(_) => Ok(InterfaceAddressIterator {
            base: addrs,
            next: addrs,
        }),
        Err(nix::Error::Sys(errno)) => Err(Error::Sys(errno)),
        Err(_) => Err(Error::Other("Failed to get network interface stats")),
    }
}

impl Drop for InterfaceAddressIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) };
    }
}

impl Iterator for InterfaceAddressIterator {
    type Item = InterfaceAddress;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddr) => {
                self.next = ifaddr.ifa_next;
                Some(InterfaceAddress::from_libc_ifaddrs(ifaddr))
            }
            None => None,
        }
    }
}

pub struct LibcReader {
    info: Vec<InterfaceInfoItem>,
}

impl LibcReader {
    pub fn new() -> Result<LibcReader> {
        let mut info = vec![];

        for addr in getifaddrs()? {
            match addr.data {
                None => continue,
                Some(_) => {
                    info.push(InterfaceInfoItem {
                        name: addr.interface_name,
                    });
                }
            }
        }

        Ok(LibcReader { info })
    }
}

impl Read for LibcReader {
    fn get_info(&self) -> &[InterfaceInfoItem] {
        &self.info
    }

    fn read(&self) -> Result<InterfaceStats> {
        let mut stats = vec![None; self.get_info().len()];

        for addr in getifaddrs()? {
            match addr.data {
                None => continue,
                Some(data) => match self.index(&addr.interface_name) {
                    None => continue,
                    Some(i) => {
                        stats[i] = Some(InterfaceStat {
                            rx: NumBytes::from(data.ifi_ibytes as u64),
                            tx: NumBytes::from(data.ifi_obytes as u64),
                        })
                    }
                },
            }
        }

        Ok(stats)
    }
}
