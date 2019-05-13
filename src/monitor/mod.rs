use crate::reader::Read;
use crate::writer::Write;
use crate::{Error, Result};

pub mod events;

pub struct Monitor {
    reader: Option<Box<dyn Read + Send>>,
    writer: Box<dyn Write>,
}

impl Monitor {
    pub fn new(reader: Box<dyn Read + Send>, writer: Box<dyn Write>) -> Monitor {
        Monitor {
            reader: Some(reader),
            writer,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let events = match self.reader.take() {
            None => return Err(Error::Other("Failed to initialize reader thread.")),
            Some(reader) => events::Events::new(reader, &mut self.writer)?,
        };

        for event in events {
            match event {
                events::Event::Tick(stats) => self.writer.update(stats)?,
                events::Event::Shutdown => break,
            }
        }

        Ok(())
    }
}
