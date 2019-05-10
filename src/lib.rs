#[macro_use]
extern crate cfg_if;

use reader::{in_libc::LibcReader, Reader};
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

pub fn run() -> Result<()> {
    let reader = LibcReader::new()?;

    println!("{:#?}", reader.get_info());
    println!("{:#?}", reader.read()?);

    Ok(())
}
