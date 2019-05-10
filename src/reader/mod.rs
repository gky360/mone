use crate::{InterfaceInfoItem, InterfaceStats};

pub mod in_libc;

pub trait Read {
    fn get_info(&self) -> &[InterfaceInfoItem];
    fn index(&self, name: &str) -> Option<usize> {
        self.get_info().iter().position(|item| item.name == name)
    }
    fn read(&self) -> InterfaceStats;
}
