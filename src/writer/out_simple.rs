use ctrlc;
use std::io;

use crate::writer::Write;
use crate::{InterfaceInfo, InterfaceStats, Result};

pub struct SimpleWriter<T: io::Write> {
    writer: T,
    prev_stats: InterfaceStats,
}

impl<T: io::Write> SimpleWriter<T> {
    pub fn new(
        mut writer: T,
        info: &InterfaceInfo,
        initial_stats: InterfaceStats,
    ) -> Result<SimpleWriter<T>> {
        writeln!(writer, "{}", info)?;
        Ok(SimpleWriter {
            writer,
            prev_stats: initial_stats,
        })
    }
}

impl<T: io::Write> Write for SimpleWriter<T> {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()> {
        ctrlc::set_handler(move || (*callback)()).expect("Failed to set Ctrl+C handler");
        Ok(())
    }

    fn update(&mut self, stats: InterfaceStats) -> Result<()> {
        let diff = InterfaceStats(
            stats
                .0
                .iter()
                .enumerate()
                .map(|(i, stat)| {
                    if let Some(stat) = stat {
                        if let Some(prev_stat) = &self.prev_stats.0[i] {
                            return Some(stat - prev_stat);
                        }
                    }
                    None
                })
                .collect(),
        );
        writeln!(self.writer, "{}", diff).unwrap_or(());
        self.prev_stats = stats;
        Ok(())
    }
}
