use std::collections::VecDeque;
use std::{io, thread};
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget};
use tui::Terminal;

use crate::writer::Write;
use crate::{InterfaceInfo, InterfaceStat, InterfaceStats, Opt, Result};

struct StatsHistory(VecDeque<InterfaceStats>);

pub struct TuiWriter {
    info: InterfaceInfo,
    n_histories: usize,
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>,
    input_thread: Option<thread::JoinHandle<()>>,
    history: StatsHistory,
    data1: Vec<(f64, f64)>,
    data2: Vec<(f64, f64)>,
}

#[derive(Clone)]
pub struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    pub fn new(interval: f64, period: f64, scale: f64) -> SinSignal {
        SinSignal {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

struct App {
    signal1: SinSignal,
    signal2: SinSignal,
    window: [f64; 2],
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

        let history = StatsHistory(
            vec![InterfaceStats(vec![None; info.0.len()]); opt.n + 1]
                .into_iter()
                .collect(),
        );
        let mut signal1 = SinSignal::new(0.2, 3.0, 18.0);
        let mut signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let data1 = signal1.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        let data2 = signal2.by_ref().take(200).collect::<Vec<(f64, f64)>>();
        Ok(TuiWriter {
            info,
            n_histories: opt.n,
            terminal,
            input_thread: None,
            history,
            data1,
            data2,
        })
    }

    fn update_history(&mut self, stats: InterfaceStats) {
        self.history.0.pop_front();
        self.history.0.push_back(stats);
    }

    fn draw(&mut self) -> Result<()> {
        let mut signal1 = SinSignal::new(0.2, 3.0, 18.0);
        let mut signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let app = App {
            signal1,
            signal2,
            window: [0.0, 20.0],
        };

        let mut datasets: Vec<Dataset>;
        for stat_name in ["rx", "tx"].iter() {
            for i in 0..self.info.0.len() {
                for t in 0..self.n_histories {}
            }
        }

        let i = 0;
        let stat_name = "rx";
        let data = self.history.0.iter().map(|stats| {
            // let stat = stats.0[i];
            // let val = match stat {};
            (0.0, 0.0)
        });

        let datasets = vec![
            Dataset::default()
                .name("data2")
                .marker(Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.data1),
            Dataset::default()
                .name("data3")
                .marker(Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.data2),
        ];

        self.terminal.draw(|mut f| {
            let size = f.size();
            Chart::default()
                .block(
                    Block::default()
                        .title("Chart")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("X Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds(app.window)
                        .labels(&[
                            &format!("{}", app.window[0]),
                            &format!("{}", (app.window[0] + app.window[1]) / 2.0),
                            &format!("{}", app.window[1]),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds([-20.0, 20.0])
                        .labels(&["-20", "0", "20"]),
                )
                .datasets(&datasets)
                .render(&mut f, size);
        })?;

        Ok(())
    }
}

impl Write for TuiWriter {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()> {
        let input_thread = thread::spawn(move || {
            let stdin = io::stdin();
            for event in stdin.keys() {
                match event {
                    Ok(key) => match key {
                        Key::Ctrl('c') | Key::Char('q') => {
                            (*callback)();
                            break;
                        }
                        _ => {}
                    },
                    Err(_) => {}
                }
            }
        });
        self.input_thread = Some(input_thread);

        Ok(())
    }

    fn update(&mut self, stats: InterfaceStats) -> Result<()> {
        self.update_history(stats);
        self.draw()?;
        Ok(())
    }
}

impl Drop for TuiWriter {
    fn drop(&mut self) {
        if let Some(thread) = self.input_thread.take() {
            thread.join().expect("Failed to shutdown tick thread");
        }
    }
}
