use std::io::Error;
use std::net::{TcpListener, TcpStream};

use crate::common::Am;
use crate::log;
use crate::logger::{Logger};
use crate::thread::ThreadPool;

pub fn start_dispatcher(
    adress: String,
    thread_pool: ThreadPool,
    logger: Am<Logger>,
    job: fn(TcpStream, Am<Logger>) -> Result<(), Error>
) -> Result<(), Error> {
    let listener = TcpListener::bind(adress)?;

    log!(logger, "DISPATCHER: Started. Available threads: {}.", thread_pool.size());

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                log!(logger, "DISPATCHER: Received a connection '{}'", stream.peer_addr()?);

                let logger_clone = logger.clone();
                thread_pool.enqueue(move || {
                    let _ = job(stream, logger_clone);
                });
            }

            Err(err) => {
                log!(logger, "*** DISPATCHER: An error occured: '{}'", err);
            }
        }
    }

    Ok(())
}
