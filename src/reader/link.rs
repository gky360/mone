// This module is inspired from
// [netlink-packet](https://docs.rs/crate/netlink-packet/0.1.1/source/src/rtnl/link/nlas/mod.rs)

use byteorder::{ByteOrder, NativeEndian};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LinkStats<T> {
    /// total packets received
    pub rx_packets: T,
    /// total packets transmitted
    pub tx_packets: T,
    /// total bytes received
    pub rx_bytes: T,
    /// total bytes transmitted
    pub tx_bytes: T,
    /// bad packets received
    pub rx_errors: T,
    /// packet transmit problems
    pub tx_errors: T,
    /// no space in linux buffers
    pub rx_dropped: T,
    /// no space available in linux
    pub tx_dropped: T,
    /// multicast packets received
    pub multicast: T,
    pub collisions: T,

    // detailed rx_errors
    pub rx_length_errors: T,
    /// receiver ring buff overflow
    pub rx_over_errors: T,
    /// received packets with crc error
    pub rx_crc_errors: T,
    /// received frame alignment errors
    pub rx_frame_errors: T,
    /// recv'r fifo overrun
    pub rx_fifo_errors: T,
    /// receiver missed packet
    pub rx_missed_errors: T,

    // detailed tx_errors
    pub tx_aborted_errors: T,
    pub tx_carrier_errors: T,
    pub tx_fifo_errors: T,
    pub tx_heartbeat_errors: T,
    pub tx_window_errors: T,

    // for cslip etc
    pub rx_compressed: T,
    pub tx_compressed: T,

    /// dropped, no handler found
    pub rx_nohandler: T,
}

const LINK_MAP_LEN: usize = 8 * 3 + 2 + 2 * 2;

pub const LINK_STATS32_LEN: usize = 24 * 4;

impl LinkStats<u32> {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < LINK_MAP_LEN {
            return Err(Error::LinkStatsError(format!(
                "IFLA_STATS is {} bytes, buffer is only {} bytes: {:#x?}",
                LINK_STATS32_LEN,
                buf.len(),
                buf
            )));
        }
        Ok(LinkStats {
            rx_packets: NativeEndian::read_u32(&buf[0..4]),
            tx_packets: NativeEndian::read_u32(&buf[4..8]),
            rx_bytes: NativeEndian::read_u32(&buf[8..12]),
            tx_bytes: NativeEndian::read_u32(&buf[12..16]),
            rx_errors: NativeEndian::read_u32(&buf[12..20]),
            tx_errors: NativeEndian::read_u32(&buf[20..24]),
            rx_dropped: NativeEndian::read_u32(&buf[24..28]),
            tx_dropped: NativeEndian::read_u32(&buf[28..32]),
            multicast: NativeEndian::read_u32(&buf[32..36]),
            collisions: NativeEndian::read_u32(&buf[36..40]),
            rx_length_errors: NativeEndian::read_u32(&buf[40..44]),
            rx_over_errors: NativeEndian::read_u32(&buf[44..48]),
            rx_crc_errors: NativeEndian::read_u32(&buf[48..52]),
            rx_frame_errors: NativeEndian::read_u32(&buf[52..56]),
            rx_fifo_errors: NativeEndian::read_u32(&buf[56..60]),
            rx_missed_errors: NativeEndian::read_u32(&buf[60..64]),
            tx_aborted_errors: NativeEndian::read_u32(&buf[64..68]),
            tx_carrier_errors: NativeEndian::read_u32(&buf[68..72]),
            tx_fifo_errors: NativeEndian::read_u32(&buf[72..76]),
            tx_heartbeat_errors: NativeEndian::read_u32(&buf[76..80]),
            tx_window_errors: NativeEndian::read_u32(&buf[80..84]),
            rx_compressed: NativeEndian::read_u32(&buf[84..88]),
            tx_compressed: NativeEndian::read_u32(&buf[88..92]),
            rx_nohandler: NativeEndian::read_u32(&buf[92..96]),
        })
    }
}
