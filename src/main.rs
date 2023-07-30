use std::env::args;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, Builder};
use std::time::Duration;

extern crate toiletcli;

use toiletcli::colors::*;
use toiletcli::common::name_from_path;
use toiletcli::flags;
use toiletcli::flags::{parse_flags, Flag, FlagType};

mod common;
mod http;
mod music;
mod server;

#[cfg(test)]
mod tests;

use common::logger::{Log, Logger};

use server::dispatcher::start_dispatcher;
use server::router::route;

use music::index::init_music_index;
use music::index::make_index;

use crate::common::logger::Verbosity;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[inline(always)]
fn warn_unstable() {
    eprintln!("{}Running Zest {}. The design is not final, and may be subject to change.",
              Style::Bold, VERSION);
    eprintln!("To report a bug, please open up an issue at <{}https://github.com/toiletbril/zest{}>.{}\n",
              Style::Underlined, Style::ResetUnderline, Style::Reset);
}

#[inline(always)]
fn eheaderln(message: &str) {
    eprintln!("{}{}{}", Color::Green, message, Color::Reset);
}

pub const DEFAULT_ADDRESS: &str = "0.0.0.0";
pub const DEFAULT_PORT: u32 = 6969;
pub const DEFAULT_THREAD_COUNT: usize = 8;
pub const DEFAULT_UTC: i8 = 0;
pub const DEFAULT_VERBOSITY: u8 = 0;

fn entry() -> Result<(), String> {
    let mut args = args();
    let program_name = name_from_path(&args.next().expect("Path should be provided"));

    let mut port_flag;
    let mut address_flag;
    let mut thread_count_flag;
    let mut utc_flag;
    let mut log_file_flag;
    let mut verbosity_flag;

    let mut show_version;
    let mut show_help;

    let mut flags: Vec<Flag> = flags!(
        show_help: BoolFlag,           ["--help"],
        show_version: BoolFlag,        ["--version"],
        thread_count_flag: StringFlag, ["-t", "--threads"],
        utc_flag: StringFlag,          ["-u", "--utc"],
        port_flag: StringFlag,         ["-p", "--port"],
        address_flag: StringFlag,      ["-a", "--address"],
        log_file_flag: BoolFlag,       ["-l", "--log-file"],
        verbosity_flag: RepeatFlag,    ["-v", "--verbose"]
    );

    let args = parse_flags(&mut args, &mut flags)?;

    let address = address_flag.is_empty()
        .then_some(DEFAULT_ADDRESS.to_owned())
        .unwrap_or(address_flag);
    let port = port_flag
        .parse::<u32>()
        .unwrap_or(DEFAULT_PORT);
    let thread_count = thread_count_flag
        .parse::<usize>()
        .unwrap_or(DEFAULT_THREAD_COUNT);
    let utc = utc_flag
        .parse::<i8>()
        .unwrap_or(DEFAULT_UTC);
    let verbosity: Verbosity =
        (verbosity_flag as u8)
        .into();

    if show_help && args.len() < 1 {
        eheaderln("USAGE");
        eprintln!("    {} [-options] <subcommand>", program_name);
        eprintln!("    Music-streaming web-server.");
        eprintln!("");
        eheaderln("SUBCOMMANDS");
        eprintln!("    serve [-ptaulvv] <index file>\tServe the music.");
        eprintln!("    index [-v]       <directory> \tIndex directory and make an index file.");
        eprintln!("");
        eheaderln("OPTIONS");
        eprintln!("    --help                       \tGet help for a subcommand.");
        eprintln!("    --version                    \tDisplay version.");
        eprintln!("");
        eprintln!("To report a bug, please open up an issue at <{}https://github.com/toiletbril/zest{}>.",
                  Style::Underlined, Style::ResetUnderline);

        return Ok(());
    }

    if show_version {
        println!("Zest {}", VERSION);
        println!("(c) toiletbril <https://github.com/toiletbril>");
        return Ok(());
    }

    if args.len() < 1 {
        return Err("Not enough arguments".into());
    }

    match args[0].as_str() {
        "serve" => {
            if show_help {
                eheaderln("USAGE");
                eprintln!("    {} serve [-options] <index file>", program_name);
                eprintln!("    Serve the music, using index file.");
                eprintln!("");
                eheaderln("OPTIONS");
                eprintln!("    -p, --port <port>      \tSet server's port.");
                eprintln!("    -a, --address <adress> \tSet server's address.");
                eprintln!("    -t, --threads <count>  \tAmount of threads to create.");
                eprintln!("    -u, --utc <hours>      \tUTC adjustment for logger.");
                eprintln!("    -l, --log-file         \tCreate a log file.");
                eprintln!("    -v[v]                  \tLogging verbosity.");
                eprintln!("        --help             \tDisplay this message.");

                return Ok(());
            }

            if args.len() < 2 {
                return Err("Not enough arguments".into());
            }

            warn_unstable();

            init_music_index(args[1].to_owned())?;

            let logger = Arc::new(Mutex::new(Logger::new(utc, log_file_flag, Verbosity::from(verbosity))));
            let logger_clone = logger.clone();

            log!(logger, "Starting the dispatcher ({} threads)...", thread_count);

            Builder::new()
                .name("dispatcher".to_string())
                .spawn(move || {
                    let _ = start_dispatcher(
                        format!("{address}:{port}"),
                        thread_count,
                        logger_clone,
                        route,
                    );
                }).map_err(|err| err.to_string())?;

            log!(logger, "Starting the logger (mode: {}, logfile: {}, {} hour offset)...",
                 verbosity, log_file_flag, utc);

            loop {
                let _ = flush!(logger);
                sleep(Duration::from_micros(1000));
            }
        }
        "index" => {
            if show_help {
                eheaderln("USAGE");
                eprintln!("    {} index [-options] <music directory>", program_name);
                eprintln!("    Index the directory and generate an index.");
                eprintln!("");
                eheaderln("OPTIONS");
                eprintln!("    -v        \tVerbose output.");
                eprintln!("        --help\tDisplay this message.");

                return Ok(());
            }

            if args.len() < 2 {
                return Err("Not enough arguments".into());
            }

            let path = args[1].to_owned();

            eprintln!("Traversing '{}'...", path);

            match make_index(&path, verbosity) {
                Err(err) => return Err(format!("While indexing '{}': {}", path, err)),
                Ok(filename) => {
                    eprintln!("Successfully traversed '{}', created '{}'.", path, filename)
                }
            }

            Ok(())
        }
        _ => {
            Err(format!("Unknown subcommand '{}'", args[0]))
        }
    }
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}ERROR{}: {}. Try using '--help' for more information.", Color::Red, Color::Reset, err);
            ExitCode::FAILURE
        }
    }
}
