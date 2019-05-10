use crate::writer::Write;
use crate::{Error, InterfaceInfo, InterfaceStats, Result};
use std::io;

pub struct SimpleWriter<T: io::Write> {
    writer: T,
}

impl<T: io::Write> SimpleWriter<T> {
    pub fn new(mut writer: T, info: &InterfaceInfo) -> Result<SimpleWriter<T>> {
        writeln!(writer, "{}", info).map_err(|_| Error::Other("Failed to output"))?;
        Ok(SimpleWriter { writer })
    }
}

impl<T: io::Write> Write for SimpleWriter<T> {
    fn update(&mut self, stats: InterfaceStats) {
        writeln!(self.writer, "{}", stats).unwrap_or(());
    }
}
