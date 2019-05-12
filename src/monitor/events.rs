use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::reader::Read;
use crate::InterfaceStats;

pub enum Event {
    Tick(InterfaceStats),
}

pub struct Events {
    running: Arc<AtomicBool>,
    rx: mpsc::Receiver<Event>,
    tick_thread: Option<thread::JoinHandle<()>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub tick_interval: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_interval: Duration::from_millis(1000),
        }
    }
}

impl Events {
    pub fn new<R: 'static + Read + Send>(reader: R) -> Events {
        Events::with_config(reader, Config::default())
    }

    pub fn with_config<R: 'static + Read + Send>(reader: R, config: Config) -> Events {
        let running = Arc::new(AtomicBool::new(true));
        {
            let running = Arc::clone(&running);
            ctrlc::set_handler(move || {
                running.store(false, Ordering::SeqCst);
            })
            .expect("Failed to set Ctrl+C handler");
        }

        let (tx, rx) = mpsc::channel();

        let tick_thread = {
            let running = Arc::clone(&running);
            let tx = mpsc::Sender::clone(&tx);
            let started_at = Instant::now();
            thread::spawn(move || {
                for t in 1.. {
                    let next_at = started_at + t * config.tick_interval;
                    thread::sleep(next_at - Instant::now());
                    let stats = reader.read();
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Err(_) = tx.send(Event::Tick(stats)) {
                        return;
                    };
                }
            })
        };

        Events {
            running,
            rx,
            tick_thread: Some(tick_thread),
        }
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
            thread.join().unwrap();
        }
    }
}
