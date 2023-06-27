use std::env::args;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, Builder};
use std::time::Duration;

extern crate toiletcli;
use toiletcli::common::name_from_path;
use toiletcli::flags;
use toiletcli::flags::{parse_flags, Flag, FlagType};

mod common;
mod dispatcher;
mod http;
mod logger;
mod music;
mod thread;
mod router;

use dispatcher::start_dispatcher;
use logger::Logger;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_ADDRESS: &str = "localhost";
const DEFAULT_PORT: u32 = 6969;
const DEFAULT_THREAD_COUNT: usize = 8;
const DEFAULT_UTC: u64 = 0;

fn entry() -> Result<(), String> {
    let mut args = args();
    let program_name = name_from_path(&args.next().expect("Path should be provided"));

    let mut port_flag;
    let mut address_flag;
    let mut thread_count_flag;
    let mut utc_flag;

    let mut show_help;

    let mut flags: Vec<Flag> = flags!(
        utc_flag: StringFlag,
        ["-u", "--utc"],
        thread_count_flag: StringFlag,
        ["-t", "--threads"],
        show_help: BoolFlag,
        ["-?", "--help"],
        port_flag: StringFlag,
        ["-p", "--port"],
        address_flag: StringFlag,
        ["-a", "--address"]
    );

    let args = parse_flags(&mut args, &mut flags)?;

    let address = if address_flag.is_empty() {
        DEFAULT_ADDRESS.to_string()
    } else {
        address_flag
    };

    let port = port_flag.parse::<u32>().unwrap_or(DEFAULT_PORT);
    let thread_count = thread_count_flag
        .parse::<usize>()
        .unwrap_or(DEFAULT_THREAD_COUNT);
    let utc = utc_flag.parse::<u64>().unwrap_or(DEFAULT_UTC);

    if show_help {
        println!("USAGE: {} [-options] <subcommand>", program_name);
        println!("Multipurpose web-server.");
        println!("");
        println!("SUBCOMMANDS:  serve <directory> \tServe the directory. Does not do anything YET.");
        println!("");
        println!("OPTIONS:      -p, --port <port>      \tSet server's port.");
        println!("              -a, --address <adress> \tSet server's address.");
        println!("              -t, --threads <count>  \tThreads to create.");
        println!("              -u, --utc <hours>      \tUTC adjustment for logger.");
        println!("");
        println!("{} (c) toiletbril <https://github.com/toiletbril>", VERSION);
        return Ok(());
    }

    if args.len() < 1 {
        return Err("Not enough arguments\nTry '--help' for more information.".to_string());
    }

    println!("Running Zest web-server, version {} (c) toiletbril <https://github.com/toiletbril>", VERSION);

    match args[0].as_str() {
        "serve" => {
            let logger = Arc::new(Mutex::new(Logger::new(utc)));
            let logger_clone = logger.clone();

            log!(logger, "Starting the dispatcher...");

            let _ = Builder::new()
                .name("dispatcher".to_string())
                .spawn(move || {
                    let _ = start_dispatcher(
                        format!("{address}:{port}"),
                        thread_count,
                        logger_clone,
                        router::route,
                    );
                });

            log!(logger, "Starting the logger...");

            loop {
                let _ = flush!(logger);
                sleep(Duration::from_micros(100));
            }
        }
        _ => Err(format!("Unknown command '{}'", args[1])),
    }
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ERROR: {}.", err);
            ExitCode::FAILURE
        }
    }
}
