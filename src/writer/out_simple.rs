use crate::writer::Write;
use crate::InterfaceStats;
use std::io;

pub struct SimpleWriter<T: io::Write> {
    writer: T,
}

impl<T: io::Write> SimpleWriter<T> {
    pub fn new(writer: T) -> SimpleWriter<T> {
        SimpleWriter { writer }
    }
}

impl<T: io::Write> Write for SimpleWriter<T> {
    fn update(&mut self, stats: InterfaceStats) {
        write!(self.writer, "{}", stats).unwrap_or(());
    }
}
