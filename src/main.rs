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
mod http;
mod music;
mod server;

use common::logger::{Log, Logger};

use server::dispatcher::start_dispatcher;
use server::router::route;

use music::index::init_music_index;
use music::index::make_index;

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
    let mut log_file_flag;

    let mut show_version;
    let mut show_help;

    let mut flags: Vec<Flag> = flags!(
        show_help: BoolFlag,           ["--help"],
        show_version: BoolFlag,        ["--version"],
        thread_count_flag: StringFlag, ["-t", "--threads"],
        utc_flag: StringFlag,          ["-u", "--utc"],
        port_flag: StringFlag,         ["-p", "--port"],
        address_flag: StringFlag,      ["-a", "--address"],
        log_file_flag: BoolFlag,       ["-l", "--log-file"]
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
        .parse::<u64>()
        .unwrap_or(DEFAULT_UTC);

    if show_help {
        println!("USAGE: {} [-options] [subcommand]", program_name);
        println!("Music-streaming web-server.");
        println!("");
        println!("SUBCOMMANDS:  serve <index file>     \tServe the music.");
        println!("              index <directory>      \tIndex directory and make an index file.");
        println!("");
        println!("OPTIONS:      -p, --port <port>      \tSet server's port.");
        println!("              -a, --address <adress> \tSet server's address.");
        println!("              -t, --threads <count>  \tThreads to create.");
        println!("              -u, --utc <hours>      \tUTC adjustment for logger.");
        println!("              -l, --log-file         \tWrite logs to a log file.");
        println!("                  --help             \tDisplay this message.");
        println!("                  --version          \tDisplay version.");
        return Ok(());
    }

    if show_version {
        println!("Zest {} (c) toiletbril <https://github.com/toiletbril>", VERSION);
        return Ok(());
    }

    if args.len() < 1 {
        return Err("Not enough arguments.\n".to_string());
    }

    println!("Running Zest {}", VERSION);

    match args[0].as_str() {
        "serve" => {
            if args.len() < 2 {
                return Err(format!("Not enough arguments.\nUSAGE: {} serve <index file>\n",
                                   program_name));
            }

            init_music_index(args[1].to_owned())?;

            let logger = Arc::new(Mutex::new(Logger::new(utc, log_file_flag)));
            let logger_clone = logger.clone();

            log!(logger, "Starting the dispatcher...");

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

            log!(logger, "Starting the logger...");

            loop {
                let _ = flush!(logger);
                sleep(Duration::from_micros(100));
            }
        }
        "index" => {
            if args.len() < 2 {
                    return Err(format!("Not enough arguments.\nUSAGE: {} index <directory>\n",
                                       program_name));
            }

            let config = args[1].to_owned();
            println!("Traversing '{}'...", config);

            match make_index(config.to_owned()) {
                Err(err) => return Err(format!("Could not index '{}': {}", config, err)),
                Ok(filename) => {
                    println!("Successfully traversed '{}', generated index file '{}'.",
                             config, filename)
                }
            }

            Ok(())
        }
        _ => {
            Err(format!("Unknown subcommand '{}'.\n",
                args[0]))
        }
    }
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ERROR: {}.\nTry '--help' for more information", err);
            ExitCode::FAILURE
        }
    }
}
