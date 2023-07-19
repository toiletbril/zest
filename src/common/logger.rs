use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::thread::current;
use std::time::SystemTime;

/// This filters out complex messages if not needed.
#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Verbosity {
    Default,
    Details,
    Debug,
}

impl From<u8> for Verbosity {
    fn from(val: u8) -> Self {
        match val {
            0 => Verbosity::Default,
            1 => Verbosity::Details,
            2 => Verbosity::Debug,
            _ => {
                Verbosity::Debug
            }
        }
    }
}

struct LogEntry {
    message: String,
    verbosity: Verbosity
}

type LogQueue = Vec<LogEntry>;

pub trait Log {
    /// Push a message into log queue.
    fn log(&mut self, verbosity: Verbosity, message: String);
    /// Flush the contents of the log queue and clear it.
    fn flush(&mut self) -> Result<(), Error>;
    fn verbosity(&self) -> Verbosity;
}

/// Quickly flush the logger behind Arc<Mutex>.
#[macro_export]
macro_rules! flush {
    ($logger:ident) => {
        if let Ok(mut $logger) = $logger.lock() {
            let _ = $logger.flush();
        }
    };
}

/// Generate a string with current UTC time.
#[macro_export]
macro_rules! time {
    ($utc:expr) => {{
        let mut _duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Should be able to get time")
            .as_secs();

        if $utc > 0 {
            _duration += $utc as u64 * 3600
        } else {
            _duration -= ($utc * -1) as u64 * 3600
        }

        let _secs = _duration % 60;
        let _mins = (_duration / 60) % 60;
        let _hrs = (_duration / 3600) % 24;

        format!("{:02}:{:02}:{:02}", _hrs, _mins, _secs)
    }};
}

/// Quickly lock and push a message into the logger behind Arc<Mutex>.
#[macro_export]
macro_rules! log {
    ($logger:expr, $verbosity:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            logger.log($verbosity, format!($($msg),*));
        }
    };
}

/// Log if verbosity matches.
#[macro_export]
macro_rules! log_matching_verbosity {
    ($logger:expr, $verbosity:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            if logger.verbosity() == $verbosity {
                logger.log($verbosity, format!($($msg),*));
            }
        }
    };
}

pub struct Logger {
    index: u64,
    queue: LogQueue,
    hour_offset: i8,
    log_file: Option<File>,
    use_verbosity: Verbosity,
}

impl Logger {
    pub fn new(utc: i8, use_file: bool, verbosity: Verbosity) -> Self {
        let mut i = 0;

        let file = use_file.then(|| {
            while Path::new(format!("./zest-{}.log", i).as_str()).exists() {
                i += 1;
            }

            let file = File::create(format!("./zest-{}.log", i));

            if let Err(err) = file {
                panic!("*** An error occured while creating a file for the logger: {}",
                      err);
            }

            file.unwrap()
        });

        return Logger {
            index: 1,
            queue: vec![],
            hour_offset: utc,
            log_file: file,
            use_verbosity: verbosity
        };
    }
}

impl Log for Logger {
    fn log(&mut self, verbosity: Verbosity, message: String) {
        let message =
            format!("{} [{}] {:?} -> {}: {}",
                    self.index, time!(self.hour_offset), current().id(),
                    current().name().map_or("MAIN".to_string(), |x| x.to_uppercase()),
                    message);

        self.queue.push(LogEntry { message: message, verbosity: verbosity });

        self.index += 1;
    }

    fn flush(&mut self) -> Result<(), Error> {
        if !&self.queue.is_empty() {
            for entry in &self.queue {
                if entry.verbosity as u8 <= self.use_verbosity as u8 {

                    println!("{}", entry.message);
                }

                self.log_file.as_ref()
                    .map(|mut x| write!(x, "{}\n", entry.message));
            }

            self.queue.clear();
        }

        Ok(())
    }

    fn verbosity(&self) -> Verbosity {
        self.use_verbosity
    }
}
