use std::fmt::Display;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::thread::current;
use std::time::SystemTime;

#[repr(u8)]
#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub enum Verbosity {
    Default,
    Details,
    Debug,
}

impl Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut verbosity_string = format!("{:?}", self);
        verbosity_string.make_ascii_lowercase();

        write!(f, "{}", verbosity_string)
    }
}

impl From<u8> for Verbosity {
    fn from(val: u8) -> Self {
        match val {
            0 => Verbosity::Default,
            1 => Verbosity::Details,
            2 => Verbosity::Debug,
            _ => Verbosity::Debug,
        }
    }
}

struct LogEntry {
    message: String,
}

type LogQueue = Vec<LogEntry>;

pub trait Log {
    /// Push a message into log queue.
    fn log(&mut self, message: String);
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
    ($logger:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            logger.log(format!($($msg),*));
        }
    };
}

// TODO: Get rid of these.

#[macro_export]
/// Logs if verbosity is greater or equal to specified.
macro_rules! log_geq {
    ($logger:expr, $verbosity_pattern:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            if logger.verbosity() as u8 >= $verbosity_pattern as u8 {
                logger.log(format!($($msg),*));
            }
        }
    };
}

#[macro_export]
/// Logs if verbosity is equal to specified.
macro_rules! log_eq {
    ($logger:expr, $verbosity_pattern:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            if logger.verbosity() as u8 == $verbosity_pattern as u8 {
                logger.log(format!($($msg),*));
            }
        }
    };
}

#[macro_export]
/// Logs if verbosity is less or equal to specified.
macro_rules! log_leq {
    ($logger:expr, $verbosity_pattern:expr, $($msg:expr),*) => {
        if let Ok(mut logger) = $logger.lock() {
            if logger.verbosity() as u8 <= $verbosity_pattern as u8 {
                logger.log(format!($($msg),*));
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
                panic!(
                    "*** An error occured while creating a file for the logger: {}",
                    err
                );
            }

            file.unwrap()
        });

        return Logger {
            index: 1,
            queue: vec![],
            hour_offset: utc,
            log_file: file,
            use_verbosity: verbosity,
        };
    }
}

impl Log for Logger {
    fn log(&mut self, message: String) {
        let time = time!(self.hour_offset);

        let thread = current();
        let thread_id = thread.id();
        let thread_name = thread
            .name()
            .map_or("MAIN".to_string(), |x| x.to_uppercase());

        let message = format!(
            "{} [{}] {:?} -> {}: {}",
            self.index, time, thread_id, thread_name, message
        );

        let entry = LogEntry {
            message: message,
        };

        self.queue.push(entry);

        self.index += 1;
    }

    fn flush(&mut self) -> Result<(), Error> {
        if !&self.queue.is_empty() {
            for entry in &self.queue {
                println!("{}", entry.message);

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
