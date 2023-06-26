use std::io::Error;
use std::thread::current;
use std::time::SystemTime;

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
    ($utc:expr) => {{
        let _duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Should be able to get time")
            .as_secs() + $utc * 3600;
        let _secs = _duration % 60;
        let _mins = (_duration / 60) % 60;
        let _hrs = (_duration / 3600) % 24;
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
    utc: u64,
}

impl Logger {
    pub fn new(utc: u64) -> Self {
        return Logger {
            index: 1,
            queue: vec![],
            utc: utc,
        };
    }

    pub fn log(&mut self, message: String) {
        self.queue.push(format!(
            "{} [{}] {:?} > {}: {}",
            self.index,
            time!(self.utc),
            current().id(),
            current().name().map_or("MAIN".to_string(), |x| x.to_uppercase()),
            message
        ));
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
