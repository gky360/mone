use ctrlc;
use std::io;

use crate::writer::Write;
use crate::{InterfaceInfo, InterfaceStats, Result};

pub struct SimpleWriter<T: io::Write> {
    writer: T,
}

impl<T: io::Write> SimpleWriter<T> {
    pub fn new(mut writer: T, info: &InterfaceInfo) -> Result<SimpleWriter<T>> {
        writeln!(writer, "{}", info)?;
        Ok(SimpleWriter { writer })
    }
}

impl<T: io::Write> Write for SimpleWriter<T> {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()> {
        ctrlc::set_handler(move || (*callback)()).expect("Failed to set Ctrl+C handler");
        Ok(())
    }

    fn update(&mut self, stats: InterfaceStats) {
        writeln!(self.writer, "{}", stats).unwrap_or(());
    }
}
