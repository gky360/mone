use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub enum Event {
    Tick(usize),
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
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
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
            thread::spawn(move || {
                for t in 0.. {
                    if let Err(_) = tx.send(Event::Tick(t)) {
                        return;
                    };
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(config.tick_interval);
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
