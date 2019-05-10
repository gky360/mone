use crate::InterfaceStats;

pub mod out_simple;

pub trait Write {
    fn update(&mut self, stats: InterfaceStats);
}
