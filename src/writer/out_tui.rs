use slice_deque::{sdeq, SliceDeque};
use std::collections::HashMap;
use std::sync::Mutex;
use std::{fmt, io, thread};
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget};
use tui::Terminal;

use crate::utils::NumBytes;
use crate::writer::Write;
use crate::{InterfaceInfo, InterfaceStats, Opt, Result};

#[derive(Clone, Debug, PartialEq)]
struct MetricHistory {
    data: Vec<SliceDeque<(f64, f64)>>,
}

impl<'a> MetricHistory {
    fn empty(info: &'a InterfaceInfo, n_histories: usize) -> MetricHistory {
        MetricHistory {
            data: vec![sdeq![(0.0,0.0); n_histories]; info.0.len()],
        }
    }

    fn get_data(&'a self, index: usize) -> &'a [(f64, f64)] {
        &self.data[index]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Metric {
    Rx,
    Tx,
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Metric::Rx => write!(f, "rx"),
            Metric::Tx => write!(f, "tx"),
        }
    }
}

impl Metric {
    fn variants() -> [Metric; 2] {
        [Metric::Rx, Metric::Tx]
    }
}

#[derive(Clone, Debug, PartialEq)]
struct History {
    current: i32,
    data: HashMap<Metric, MetricHistory>,
}

impl<'a> History {
    fn empty(info: &'a InterfaceInfo, n_histories: usize) -> History {
        History {
            current: 0,
            data: Metric::variants()
                .iter()
                .map(|&metric| (metric, MetricHistory::empty(info, n_histories)))
                .collect(),
        }
    }

    fn get_data(&'a self, metric: Metric, index: usize) -> &'a [(f64, f64)] {
        self.data
            .get(&metric)
            .expect("Could not find a metric.")
            .get_data(index)
    }

    fn pop_front(&mut self) {
        for (_, h) in self.data.iter_mut() {
            for d in h.data.iter_mut() {
                d.pop_front();
            }
        }
    }

    fn push_back(&mut self, diff: InterfaceStats) {
        self.current += 1;
        for (metric, h) in self.data.iter_mut() {
            for (i, d) in h.data.iter_mut().enumerate() {
                let num = match &diff.0[i] {
                    Some(stat) => match metric {
                        Metric::Rx => stat.rx,
                        Metric::Tx => stat.tx,
                    },
                    None => 0.into(),
                };
                let val = match num.to_f64() {
                    Some(val) => val.max(1.0).log2(),
                    None => 0.0,
                };
                d.push_back((f64::from(self.current), val));
            }
        }
    }

    fn push_back_pop_front(&mut self, diff: InterfaceStats) {
        self.pop_front();
        self.push_back(diff);
    }
}

type TuiBackend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>;

lazy_static! {
    static ref TERMINAL: Mutex<Terminal<TuiBackend>> = {
        let terminal = {
            let stdout = io::stdout()
                .into_raw_mode()
                .expect("Failed to turn stdout into raw mode");
            let stdout = MouseTerminal::from(stdout);
            let stdout = AlternateScreen::from(stdout);
            let backend = TermionBackend::new(stdout);
            let mut terminal = Terminal::new(backend).expect("Failed to setup terminal backend");
            terminal.hide_cursor().expect("Failed to hide cursor");
            terminal
        };
        Mutex::new(terminal)
    };
}

pub struct TuiWriter {
    info: InterfaceInfo,
    n_histories: usize,
    input_thread: Option<thread::JoinHandle<()>>,
    prev_stats: InterfaceStats,
    history: History,
}

impl TuiWriter {
    const Y_WINDOW: [f64; 2] = [0.0, 30.0]; // 1 B -- 1GiB in log scale

    fn get_y_labels() -> [String; 4] {
        [
            format!("{}", NumBytes::from((1024 as u64).pow(0))),
            format!("{}", NumBytes::from((1024 as u64).pow(1))),
            format!("{}", NumBytes::from((1024 as u64).pow(2))),
            format!("{}", NumBytes::from((1024 as u64).pow(3))),
        ]
    }

    pub fn new(
        opt: &Opt,
        info: &InterfaceInfo,
        initial_stats: InterfaceStats,
    ) -> Result<TuiWriter> {
        let history = History::empty(&info, opt.n);
        let info = info.clone();

        Ok(TuiWriter {
            info,
            n_histories: opt.n,
            input_thread: None,
            prev_stats: initial_stats,
            history,
        })
    }

    fn update_history(&mut self, stats: InterfaceStats) {
        let diff = &stats - &self.prev_stats;
        self.prev_stats = stats;
        self.history.push_back_pop_front(diff);
    }

    fn draw(&self) -> Result<()> {
        let mut terminal = TERMINAL.lock().expect("Failed to aquire lock");
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());
            let datasets: Vec<Vec<Dataset>> = Metric::variants()
                .iter()
                .map(|&metric| {
                    self.info
                        .0
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            Dataset::default()
                                .name(&item.name)
                                .marker(Marker::Dot)
                                .style(Style::default().fg(Color::Cyan))
                                .data(self.history.get_data(metric, i))
                        })
                        .collect()
                })
                .collect();
            let y_labels = Self::get_y_labels();
            for (l, metric) in Metric::variants().iter().enumerate() {
                Chart::default()
                    .block(
                        Block::default()
                            .title(&format!("{}", metric))
                            .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        Axis::default()
                            .title("time")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([
                                f64::from(self.history.current - self.n_histories as i32),
                                f64::from(self.history.current),
                            ])
                            .labels(&[""]),
                    )
                    .y_axis(
                        Axis::default()
                            .title("Bytes/s")
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds(Self::Y_WINDOW)
                            .labels(&y_labels),
                    )
                    .datasets(&datasets[l])
                    .render(&mut f, chunks[l]);
            }
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
