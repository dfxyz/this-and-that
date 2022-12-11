use std::{
    io::{stdout, Stdout, Write},
    thread::{spawn, JoinHandle},
};

use chrono::Local;
use crossbeam_channel::{Receiver, Sender};
use log::{LevelFilter, Log, Metadata, Record};

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3f%z";

pub fn init(level: LevelFilter) -> LoggerHandle {
    let (tx, rx) = crossbeam_channel::unbounded::<LoggerMessage>();
    let tx = Box::leak(Box::new(tx));

    let backend = LoggerBackend {
        rx,
        stdout: stdout(),
    };
    let join_handle = spawn(move || {
        backend.run();
    });

    let frontend = LoggerFrontend { tx, level };
    let frontend = Box::leak(Box::new(frontend));
    log::set_max_level(level);
    log::set_logger(frontend).unwrap();

    LoggerHandle {
        tx,
        join_handle: Some(join_handle),
    }
}

enum LoggerMessage {
    Log(String),
    Shutdown,
}

pub struct LoggerHandle {
    tx: &'static Sender<LoggerMessage>,
    join_handle: Option<JoinHandle<()>>,
}
impl Drop for LoggerHandle {
    fn drop(&mut self) {
        self.tx.send(LoggerMessage::Shutdown).unwrap();
        self.join_handle.take().unwrap().join().unwrap();
    }
}

struct LoggerFrontend {
    tx: &'static Sender<LoggerMessage>,
    level: LevelFilter,
}

impl Log for LoggerFrontend {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let s = format!(
            "{datetime}|{level}|{file}:{line}|{args}\n",
            datetime = Local::now().format(DATETIME_FORMAT),
            level = record.level(),
            file = record.file().unwrap_or("<unknown>"),
            line = record.line().unwrap_or(0),
            args = record.args(),
        );
        let _ = self.tx.send(LoggerMessage::Log(s));
    }

    fn flush(&self) {}
}

struct LoggerBackend {
    rx: Receiver<LoggerMessage>,
    stdout: Stdout,
}
impl LoggerBackend {
    fn run(mut self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                LoggerMessage::Log(s) => {
                    if self.stdout.write_all(s.as_bytes()).is_err() {
                        return;
                    }
                }
                LoggerMessage::Shutdown => {
                    return;
                }
            }
        }
    }
}
