use std::{
    io::{stdout, Stdout, Write},
    thread::{spawn, JoinHandle},
};

use chrono::Local;
use crossbeam_channel::{Receiver, Sender};
use log::{LevelFilter, Log, Metadata, Record};

pub fn init(level: LevelFilter, formatter: Option<fn(&Record) -> String>) -> LoggerHandle {
    let (tx, rx) = crossbeam_channel::unbounded::<LoggerMessage>();
    let tx = Box::new(tx);
    let tx = Box::leak(tx) as &Sender<LoggerMessage>;

    let backend = LoggerBackend {
        receiver: rx,
        stdout: stdout(),
    };
    let join_handle = spawn(move || {
        backend.run();
    });

    let formatter = match formatter {
        Some(x) => x,
        None => default_formatter,
    };
    let frontend = LoggerFrontend {
        sender: tx,
        level,
        formatter,
    };
    let frontend = Box::new(frontend);
    let frontend = Box::leak(frontend);
    log::set_max_level(level);
    log::set_logger(frontend).unwrap();

    LoggerHandle {
        sender: tx,
        join_handle: Some(join_handle),
    }
}

fn default_formatter(record: &Record) -> String {
    format!(
        "[{datetime}][{level}][{file}:{line}] {args}\n",
        datetime = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z"),
        level = record.level(),
        file = record.file().unwrap_or("<unknown>"),
        line = record.line().unwrap_or(0),
        args = record.args(),
    )
}

enum LoggerMessage {
    Log(String),
    Shutdown,
}

pub struct LoggerHandle {
    sender: &'static Sender<LoggerMessage>,
    join_handle: Option<JoinHandle<()>>, // None on dropping
}
impl Drop for LoggerHandle {
    fn drop(&mut self) {
        self.sender.send(LoggerMessage::Shutdown).unwrap();
        self.join_handle.take().unwrap().join().unwrap();
    }
}

struct LoggerFrontend {
    sender: &'static Sender<LoggerMessage>,
    level: LevelFilter,
    formatter: fn(&Record) -> String,
}

impl Log for LoggerFrontend {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let msg = LoggerMessage::Log((self.formatter)(record));
        let _ = self.sender.send(msg);
    }

    fn flush(&self) {}
}

struct LoggerBackend {
    receiver: Receiver<LoggerMessage>,
    stdout: Stdout,
}
impl LoggerBackend {
    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
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
