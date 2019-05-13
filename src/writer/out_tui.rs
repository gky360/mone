use std::io;

use crate::writer::Write;
use crate::{InterfaceInfo, InterfaceStats, Result};

pub struct TuiWriter<T: io::Write> {
    writer: T,
}

impl<T: io::Write> TuiWriter<T> {
    pub fn new(mut writer: T, info: &InterfaceInfo) -> Result<TuiWriter<T>> {
        writeln!(writer, "Using TUI Writer!")?;
        writeln!(writer, "{}", info)?;
        Ok(TuiWriter { writer })
    }
}

impl<T: io::Write> Write for TuiWriter<T> {
    fn update(&mut self, stats: InterfaceStats) {
        writeln!(self.writer, "{}", stats).unwrap_or(());
    }
}
