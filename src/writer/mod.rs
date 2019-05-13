use crate::{InterfaceStats, Result};

pub mod out_simple;
pub mod out_tui;

pub trait Write {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()>;
    fn update(&mut self, stats: InterfaceStats) -> Result<()>;
}
