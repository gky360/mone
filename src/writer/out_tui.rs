use std::collections::VecDeque;
use std::{io, thread};
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::writer::Write;
use crate::Opt;
use crate::{InterfaceInfo, InterfaceStat, InterfaceStats, Result};

struct StatsHistory(Vec<VecDeque<Option<InterfaceStat>>>);

pub struct TuiWriter {
    info: InterfaceInfo,
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>,
    input_thread: Option<thread::JoinHandle<()>>,
    history: StatsHistory,
}

impl TuiWriter {
    pub fn new(opt: &Opt, info: &InterfaceInfo) -> Result<TuiWriter> {
        let info = info.clone();

        let terminal = {
            let stdout = io::stdout().into_raw_mode()?;
            let stdout = MouseTerminal::from(stdout);
            let stdout = AlternateScreen::from(stdout);
            let backend = TermionBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;
            terminal
        };

        let history = StatsHistory(vec![vec![None; opt.n].into_iter().collect(); info.0.len()]);
        Ok(TuiWriter {
            info,
            terminal,
            input_thread: None,
            history,
        })
    }
}

impl Write for TuiWriter {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()> {
        let input_thread = thread::spawn(move || {
            let stdin = io::stdin();
            for event in stdin.keys() {
                match event {
                    Ok(key) => {
                        if key == Key::Ctrl('c') {
                            (*callback)();
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }
        });
        self.input_thread = Some(input_thread);

        Ok(())
    }

    fn update(&mut self, _stats: InterfaceStats) {
        // writeln!(self.writer, "{}", stats).unwrap_or(());
    }
}

impl Drop for TuiWriter {
    fn drop(&mut self) {
        if let Some(thread) = self.input_thread.take() {
            thread.join().expect("Failed to shutdown tick thread");
        }
    }
}
