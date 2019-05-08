use std::{error, fmt, result};

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
    Ok(())
}
