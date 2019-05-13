use crate::writer::Write;
use crate::{Error, InterfaceInfo, InterfaceStats, Result};
use std::io;

pub struct TuiWriter<T: io::Write> {
    writer: T,
}

impl<T: io::Write> TuiWriter<T> {
    pub fn new(mut writer: T, info: &InterfaceInfo) -> Result<TuiWriter<T>> {
        writeln!(writer, "Using TUI Writer!").map_err(|_| Error::Other("Failed to output"))?;
        writeln!(writer, "{}", info).map_err(|_| Error::Other("Failed to output"))?;
        Ok(TuiWriter { writer })
    }
}

impl<T: io::Write> Write for TuiWriter<T> {
    fn update(&mut self, stats: InterfaceStats) {
        writeln!(self.writer, "{}", stats).unwrap_or(());
    }
}
