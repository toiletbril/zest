use std::fs::{File};
use std::io::{Error, Write};
use std::path::Path;
use std::thread::current;
use std::time::SystemTime;

pub trait Log {
    fn log(&mut self, message: String);
    fn flush(&mut self) -> Result<(), Error>;
}

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
    ($logger:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            logger.log(format!($($msg),*));
        }
    };
}

type LogQueue = Vec<String>;

pub struct Logger {
    index: usize,
    queue: LogQueue,
    utc: u64,
    file: Option<File>,
}

impl Logger {
    pub fn new(utc: u64, use_file: bool) -> Self {
        let mut i = 0;

        let file = if use_file {
            while Path::new(format!("./zest-log-{}.txt", i).as_str()).exists() {
                i += 1;
            }

            let file = File::create(format!("./zest-{}.log", i));

            if let Err(err) = file {
                panic!("*** An error occured while creating a file for the logger: {}", err);
            }

            Some(file.unwrap())
        } else {
            None
        };

        return Logger {
            index: 1,
            queue: vec![],
            utc: utc,
            file: file,
        };
    }
}

impl Log for Logger {
    fn log(&mut self, message: String) {
        self.queue.push(format!(
            "{} [{}] {:?} -> {}: {}",
            self.index,
            time!(self.utc),
            current().id(),
            current().name().map_or("MAIN".to_string(), |x| x.to_uppercase()),
            message
        ));
        self.index += 1;
    }

    fn flush(&mut self) -> Result<(), Error> {
        if !&self.queue.is_empty() {
            for entry in &self.queue {
                self.file.as_ref()
                    .map(|mut x| write!(x, "{}\n", entry));
                println!("{}", entry);
            }
            self.queue.clear();
        }

        Ok(())
    }
}
