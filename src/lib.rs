#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

use std::{fmt, io, ops, result};
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

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    IoError(io::Error),
    #[fail(display = "{}", _0)]
    NixError(nix::Error),
    #[fail(display = "Exiting")]
    Exiting,
    #[fail(display = "{}", _0)]
    Other(&'static str),
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error {
        Error::NixError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
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

impl ops::Sub for &InterfaceStat {
    type Output = InterfaceStat;
    fn sub(self, other: &InterfaceStat) -> Self::Output {
        InterfaceStat {
            rx: self.rx - other.rx,
            tx: self.tx - other.tx,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStats(Vec<Option<InterfaceStat>>);

impl InterfaceStats {
    const DELIMITER: &'static str = " | ";

    fn empty(len: usize) -> InterfaceStats {
        InterfaceStats(vec![None; len])
    }
}

impl ops::Sub for &InterfaceStats {
    type Output = InterfaceStats;
    fn sub(self, other: &InterfaceStats) -> Self::Output {
        assert_eq!(self.0.len(), other.0.len());
        InterfaceStats(
            self.0
                .iter()
                .enumerate()
                .map(|(i, stat)| {
                    if let Some(stat) = stat {
                        if let Some(other_stat) = &other.0[i] {
                            return Some(stat - other_stat);
                        }
                    }
                    None
                })
                .collect(),
        )
    }
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

    /// Number of stats history to show
    #[structopt(short = "n", default_value = "180")]
    pub n: usize,
}

pub fn run(opt: &Opt) -> Result<()> {
    let reader: Box<dyn Read + Send> = match opt.reader {
        ReaderType::libc => Box::new(LibcReader::new()?),
    };
    let writer: Box<dyn Write> = match opt.writer {
        WriterType::tui => Box::new(TuiWriter::new(&opt, reader.get_info(), reader.read())?),
        WriterType::simple => Box::new(SimpleWriter::new(
            io::stdout(),
            reader.get_info(),
            reader.read(),
        )?),
    };

    let mut monitor = Monitor::new(reader, writer);

    monitor.run()
}
