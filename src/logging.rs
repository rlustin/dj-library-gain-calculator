use chrono::Local;
use console::style;
use console::Color::{Cyan, Magenta, Red, White, Yellow};
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use log::Level::{Debug, Error, Info, Trace, Warn};
use log::{Level, Log, Metadata, Record};
use parking_lot::RwLock;
use std::io;
use std::io::Write;
use std::sync::{Arc, Weak};

lazy_static! {
    static ref PROGRESS_BAR: RwLock<Option<Weak<ProgressBar>>> = RwLock::new(None);
}

pub struct Logger;

impl Logger {
    fn get_actual_level(&self, metadata: &Metadata<'_>) -> Level {
        metadata.level()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.get_actual_level(metadata) <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = self.get_actual_level(record.metadata());
        let (level_name, level_color) = match level {
            Error => ("ERROR", Red),
            Warn => ("WARN ", Magenta),
            Info => ("INFO ", Cyan),
            Debug => ("DEBUG", Yellow),
            Trace => ("TRACE", White),
        };

        let message = format!(
            "{} {} {}",
            style(format!("  {}  ", level_name)).bg(level_color).black(),
            style(Local::now()).dim(),
            style(record.args()),
        );

        if let Some(progress_bar) = get_progress_bar() {
            progress_bar.println(message);
        } else {
            writeln!(io::stderr(), "{}", message).ok();
        }
    }

    fn flush(&self) {}
}

pub fn set_progress_bar(progress_bar: Option<Weak<ProgressBar>>) {
    *PROGRESS_BAR.write() = progress_bar;
}

fn get_progress_bar() -> Option<Arc<ProgressBar>> {
    PROGRESS_BAR.read().as_ref()?.upgrade()
}
