use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, RwLock};

use crossterm::terminal::ClearType;
use crossterm::{cursor, style, terminal, QueueableCommand};

use log4rs::append::Append;

use crate::pos::Pos;

pub struct Appender {
    start: Pos,
    max_size: usize,
    buffer: Arc<Mutex<VecDeque<String>>>,
    logging_enabled: Arc<RwLock<bool>>,
}

impl Appender {
    pub fn new(start: impl Into<Pos>, size: usize, logging_enabled: Arc<RwLock<bool>>) -> Self {
        Appender {
            start: start.into(),
            max_size: size,
            buffer: Arc::new(Mutex::new(VecDeque::default())),
            logging_enabled,
        }
    }

    pub fn display(&self, stdout: &mut io::Stdout, buffer: &VecDeque<String>) -> io::Result<()> {
        // Border
        stdout.queue(style::SetForegroundColor(style::Color::DarkGreen))?;

        for (i, line) in buffer.iter().rev().enumerate() {
            stdout
                .queue(cursor::MoveTo(self.start.x, self.start.y + 1 + i as u16))?
                .queue(style::Print(line))?
                .queue(terminal::Clear(ClearType::UntilNewLine))?;
        }

        stdout.flush()
    }
}

impl Append for Appender {
    fn append(&self, record: &log::Record) -> anyhow::Result<()> {
        let mut buffer = self
            .buffer
            .lock()
            .expect("Unable to get lock on log buffer");

        while buffer.len() >= self.max_size {
            buffer.pop_front();
        }

        let line = format!("{:?}", record.args());

        buffer.push_back(line);

        if self.logging_enabled.read().map(|e| *e).unwrap_or(false) {
            self.display(&mut std::io::stdout(), &buffer)?;
        }

        Ok(())
    }

    fn flush(&self) {}
}

impl Debug for Appender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VecAppender with {}/{} entries, output enabled: {}",
            self.max_size,
            self.buffer
                .lock()
                .expect("Unable to lock buffer to get size")
                .len(),
            self.logging_enabled.read().map(|e| *e).unwrap_or(false)
        )
    }
}
