use crate::reader::Read;
use crate::writer::Write;
use crate::{Error, Result};

pub mod events;

pub struct Monitor<R: 'static + Read + Send, W: Write> {
    reader: Option<R>,
    writer: W,
}

impl<R: 'static + Read + Send, W: Write> Monitor<R, W> {
    pub fn new(reader: R, writer: W) -> Monitor<R, W> {
        Monitor {
            reader: Some(reader),
            writer,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let events = match self.reader.take() {
            None => return Err(Error::Other("Failed to initialize reader thread.")),
            Some(reader) => events::Events::new(reader),
        };

        for event in events {
            match event {
                events::Event::Tick(stats) => {
                    self.writer.update(stats);
                }
            }
        }

        Ok(())
    }
}
