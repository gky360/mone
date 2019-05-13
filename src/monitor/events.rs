use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::reader::Read;
use crate::writer::Write;
use crate::{InterfaceStats, Result};

pub enum Event {
    Tick(InterfaceStats),
    Shutdown,
}

pub struct Events {
    running: Arc<AtomicBool>,
    rx: mpsc::Receiver<Event>,
    tick_thread: Option<thread::JoinHandle<()>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub reader_interval: Duration,
    pub tick_steps: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            reader_interval: Duration::from_millis(1000),
            tick_steps: 4,
        }
    }
}

impl Events {
    pub fn new(reader: Box<dyn Read + Send>, writer: &mut Box<dyn Write>) -> Result<Events> {
        Events::with_config(reader, writer, Config::default())
    }

    pub fn with_config(
        reader: Box<dyn Read + Send>,
        writer: &mut Box<dyn Write>,
        config: Config,
    ) -> Result<Events> {
        let (tx, rx) = mpsc::channel();

        let running = Arc::new(AtomicBool::new(true));

        let tick_thread = {
            let running = Arc::clone(&running);
            let tx = mpsc::Sender::clone(&tx);
            let started_at = Instant::now();
            let tick_interval = config.reader_interval / config.tick_steps;
            thread::spawn(move || {
                for t in 1.. {
                    let next_at = started_at + t * tick_interval;
                    thread::sleep(next_at - Instant::now());
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    if t % config.tick_steps == 0 {
                        let stats = reader.read();
                        if let Err(_) = tx.send(Event::Tick(stats)) {
                            break;
                        };
                    }
                }
            })
        };

        writer.setup_shutdown(Box::new(move || {
            if let Err(_) = tx.send(Event::Shutdown) {
                return;
            };
        }))?;

        Ok(Events {
            running,
            rx,
            tick_thread: Some(tick_thread),
        })
    }
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        match self.rx.recv() {
            Ok(event) => Some(event),
            Err(_) => None,
        }
    }
}

impl Drop for Events {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.tick_thread.take() {
            thread.join().expect("Failed to shutdown tick thread");
        }
    }
}
