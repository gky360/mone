#[macro_use]
extern crate cfg_if;

use std::{error, fmt, io, result};
use structopt::{clap::arg_enum, StructOpt};

use crate::monitor::Monitor;
use crate::reader::{in_libc::LibcReader, Read};
use crate::utils::NumBytes;
use crate::writer::{out_simple::SimpleWriter, out_tui::TuiWriter, Write};

pub mod monitor;
pub mod reader;
pub mod utils;
pub mod writer;

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

impl fmt::Display for InterfaceInfoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:<width$}",
            self.name,
            width = InterfaceStat::DISPLAY_WIDTH
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceInfo(Vec<InterfaceInfoItem>);

impl fmt::Display for InterfaceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.0.len();
        for (i, item) in self.0.iter().enumerate() {
            write!(
                f,
                "{}{}",
                item,
                if i == len - 1 {
                    ""
                } else {
                    InterfaceStats::DELIMITER
                }
            )?
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStat {
    rx: NumBytes<u64>,
    tx: NumBytes<u64>,
}

impl InterfaceStat {
    const DISPLAY_WIDTH: usize =
        NumBytes::<u64>::DISPLAY_WIDTH + 1 + NumBytes::<u64>::DISPLAY_WIDTH;
}

impl fmt::Display for InterfaceStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.rx, self.tx)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStats(Vec<Option<InterfaceStat>>);

impl InterfaceStats {
    const DELIMITER: &'static str = " | ";
}

impl fmt::Display for InterfaceStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.0.len();
        for (i, stat) in self.0.iter().enumerate() {
            match stat {
                Some(stat) => write!(f, "{}", stat)?,
                None => write!(
                    f,
                    "{:<w$} {:<w$}",
                    "None",
                    "None",
                    w = NumBytes::<u64>::DISPLAY_WIDTH
                )?,
            }
            write!(f, "{}", if i == len - 1 { "" } else { Self::DELIMITER })?
        }
        Ok(())
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(PartialEq, Debug)]
    pub enum ReaderType {
        libc,
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(PartialEq, Debug)]
    pub enum WriterType {
        tui,
        simple,
    }
}

static DEFAULT_READER: &str = "libc";
static DEFAULT_WRITER: &str = "tui";

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    /// Reader to use
    ///
    /// - libc: collect network interface stats using getifaddr from libc{n}
    #[structopt(
        short = "r",
        long = "reader",
        raw(possible_values = "&ReaderType::variants()"),
        raw(default_value = "DEFAULT_READER")
    )]
    pub reader: ReaderType,
    /// Writer to use
    ///
    /// - tui: output in TUI mode{n}- simple: output simple log to stdout{n}
    #[structopt(
        short = "w",
        long = "writer",
        raw(possible_values = "&WriterType::variants()"),
        raw(default_value = "DEFAULT_WRITER")
    )]
    pub writer: WriterType,
}

pub fn run(opt: &Opt) -> Result<()> {
    let reader: Box<dyn Read + Send> = match opt.reader {
        ReaderType::libc => Box::new(LibcReader::new()?),
    };
    let writer: Box<dyn Write> = match opt.writer {
        WriterType::tui => Box::new(TuiWriter::new(io::stdout(), reader.get_info())?),
        WriterType::simple => Box::new(SimpleWriter::new(io::stdout(), reader.get_info())?),
    };

    let mut monitor = Monitor::new(reader, writer);

    monitor.run()
}
