use crate::reader::Read;
use crate::writer::Write;
use crate::Result;

pub struct Monitor<R: Read, W: Write> {
    reader: R,
    writer: W,
}

impl<R: Read, W: Write> Monitor<R, W> {
    pub fn new(reader: R, writer: W) -> Monitor<R, W> {
        Monitor { reader, writer }
    }

    pub fn run(&mut self) -> Result<()> {
        let stats = self.reader.read();
        self.writer.update(stats);

        Ok(())
    }
}
