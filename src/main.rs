use std::env::args;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

extern crate toiletcli;
use toiletcli::flags::{Flag, FlagType, parse_flags};
use toiletcli::common::name_from_path;
use toiletcli::flags;

mod common;
mod logger;
mod dispatcher;
mod thread;
mod worker;
mod http;

use logger::{Logger};
use dispatcher::start_dispatcher;
use thread::ThreadPool;

const DEFAULT_ADDRESS: &str        = "localhost";
const DEFAULT_PORT: u32           = 6969;
const DEFAULT_THREAD_COUNT: usize = 128;

fn entry() -> Result<(), String> {
    let mut args = args();
    let program_name = name_from_path(&args.next().expect("Path should be provided"));

    let mut port_flag;
    let mut address_flag;
    let mut thread_count_flag;

    let mut show_help;

    let mut flags: Vec<Flag> = flags!(
        thread_count_flag: StringFlag, ["-t", "--threads"],
        show_help: BoolFlag,           ["-?", "--help"],
        port_flag: StringFlag,         ["-p", "--port"],
        address_flag: StringFlag,      ["-a", "--address"]
    );

    let args = parse_flags(&mut args, &mut flags)?;

    let address = if address_flag.is_empty() {
        DEFAULT_ADDRESS.to_string()
    } else {
        address_flag
    };

    let port = port_flag.parse::<u32>()
        .unwrap_or(DEFAULT_PORT);
    let thread_count = thread_count_flag.parse::<usize>()
        .unwrap_or(DEFAULT_THREAD_COUNT);

    if show_help {
        println!("USAGE: {} [-options] <subcommand>", program_name);
        println!("Web-server capable of streaming music.");
        println!("");
        println!("SUBCOMMANDS:  serve <directory>    \tServe the directory. Does not do anything YET.");
        println!("");
        println!("OPTIONS:      -p, --port <port>    \tSet server's port.   Default: '{}'", DEFAULT_PORT);
        println!("              -a, --address <adress>\tSet server's address. Default: '{}'", DEFAULT_ADDRESS);
        println!("              -t, --threads <count>\tThread count. Default: '{}'", DEFAULT_THREAD_COUNT);
        println!("");
        println!("(c) toiletbril <https://github.com/toiletbril>");
        return Ok(());
    }

    if args.len() < 1 {
        return Err("Not enough arguments\nTry '--help' for more information.".to_string());
    }

    match args[0].as_str() {
        "serve" => {
            let logger = Arc::new(Mutex::new(Logger::new()));
            log!(logger, "MAIN: Serving at <http://{address}:{port}>...");
            log!(logger, "MAIN: Starting the dispatcher...");

            let thread_pool = ThreadPool::new(thread_count);
            let logger_clone = logger.clone();

            let _ = spawn(move || {
                let _ = start_dispatcher(format!("{address}:{port}"),
                                         thread_pool, logger_clone, worker::handle_stream);
            });

            log!(logger, "MAIN: Starting log loop...");

            loop {
                let _ = flush!(logger);
                sleep(Duration::from_micros(100));
            }
        }
        _ => Err(format!("Unknown command '{}'", args[1]))
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
