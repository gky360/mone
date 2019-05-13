use crate::InterfaceStats;

pub mod out_simple;
pub mod out_tui;

pub trait Write {
    fn update(&mut self, stats: InterfaceStats);
}
