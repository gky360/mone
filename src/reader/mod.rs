use crate::utils::NumBytes;
use crate::Result;
use std::collections::HashMap;

pub mod in_libc;

pub trait Reader {
    fn read(&self) -> Result<InterfaceStats>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStat {
    rx: NumBytes<u64>,
    tx: NumBytes<u64>,
}

pub type InterfaceStats = HashMap<String, InterfaceStat>;
