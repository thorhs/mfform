use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use log4rs::append::Append;

pub(crate) struct Appender {
    max_size: usize,
    buffer: Arc<Mutex<VecDeque<String>>>,
}

impl Appender {
    pub fn with_capacity(max_size: usize) -> Self {
        Appender {
            max_size,
            buffer: Arc::new(Mutex::new(VecDeque::default())),
        }
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

        let line = format!("{:?}", record);

        buffer.push_back(line);

        Ok(())
    }

    fn flush(&self) {}
}

impl Debug for Appender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VecAppender with {}/{} entries",
            self.max_size,
            self.buffer
                .lock()
                .expect("Unable to lock buffer to get size")
                .len()
        )
    }
}
