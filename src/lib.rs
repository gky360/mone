#[macro_use]
extern crate cfg_if;

use crate::utils::NumBytes;
use reader::{in_libc::LibcReader, Read};
use std::{error, fmt, result};

pub mod reader;
pub mod utils;

pub type Errno = nix::errno::Errno;
pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(Errno),
    Other(&'static str),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Sys(ref errno) => errno.desc(),
            Error::Other(msg) => msg,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Sys(errno) => write!(f, "{:?}: {}", errno, errno.desc()),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceInfoItem {
    name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStat {
    rx: NumBytes<u64>,
    tx: NumBytes<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStats(Vec<Option<InterfaceStat>>);

pub fn run() -> Result<()> {
    let reader = LibcReader::new()?;

    println!("{:#?}", reader.get_info());
    println!("{:#?}", reader.read());

    Ok(())
}
