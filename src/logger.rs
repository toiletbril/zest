use std::io::Error;
use std::time::SystemTime;
use std::thread::{current};

#[macro_export]
macro_rules! flush {
    ($logger:ident) => {
        if let Ok(mut $logger) = $logger.lock() {
            $logger.flush()
        } else {
            unreachable!();
        }
    };
}

#[macro_export]
macro_rules! time {
    () => {{
        let _duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Should be able to get time");
        let _secs = _duration.as_secs() % 60;
        let _mins = (_duration.as_secs() / 60) % 60;
        let _hrs = (_duration.as_secs() / 3600) % 24;
        format!("{:02}:{:02}:{:02}", _hrs, _mins, _secs)
    }};
}

#[macro_export]
macro_rules! log {
    ($logger:ident, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            logger.log(format!($($msg),*));
        }
    };
}

pub trait Log {
    fn log(&self, logger: &mut Logger, message: String);
}

type LogQueue = Vec<String>;

pub struct Logger {
    index: usize,
    queue: LogQueue,
}

impl Logger {
    pub fn new() -> Self {
        return Logger {
            index: 1,
            queue: vec![],
        };
    }
}

impl Logger {
    pub fn log(&mut self, message: String) {
        self.queue
            .push(format!("{} [{}] {:?}\n> {}",
                  self.index, time!(), current().id(), message));
        self.index += 1;
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        if !&self.queue.is_empty() {
            for entry in &self.queue {
                println!("{}", entry);
            }
            self.queue.clear();
        }

        Ok(())
    }
}
