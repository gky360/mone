use std::collections::VecDeque;
use std::io;
use termion::input::MouseTerminal;
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
            history,
        })
    }
}

impl Write for TuiWriter {
    fn update(&mut self, stats: InterfaceStats) {
        // writeln!(self.writer, "{}", stats).unwrap_or(());
    }
}
