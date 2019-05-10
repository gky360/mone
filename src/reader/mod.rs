use crate::utils::NumBytes;
use crate::Result;

pub mod in_libc;

pub trait Read {
    fn get_info(&self) -> &[InterfaceInfoItem];
    fn index(&self, name: &str) -> Option<usize> {
        self.get_info().iter().position(|item| item.name == name)
    }
    fn read(&self) -> Result<InterfaceStats>;
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

pub type InterfaceStats = Vec<Option<InterfaceStat>>;
