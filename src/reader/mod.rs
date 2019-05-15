use crate::{InterfaceInfo, InterfaceStats};

pub mod in_libc;

#[cfg(target_os = "linux")]
mod link;

pub trait Read {
    fn get_info(&self) -> &InterfaceInfo;
    fn index(&self, name: &str) -> Option<usize> {
        self.get_info().0.iter().position(|item| item.name == name)
    }
    fn read(&self) -> InterfaceStats;
}
