use std::io::Error;
use std::net::{TcpListener, TcpStream};

use crate::common::Am;
use crate::log;
use crate::logger::{Logger};
use crate::thread::ThreadPool;

pub fn start_dispatcher(
    address: String,
    thread_count: usize,
    logger: Am<Logger>,
    job: fn(TcpStream, Am<Logger>) -> Result<(), Error>
) -> Result<(), Error> {
    log!(logger, "Binding to <http://{address}>...");
    let listener = TcpListener::bind(address)?;

    let thread_pool = ThreadPool::new(thread_count);
    log!(logger, "Started. Available threads: {}.", thread_pool.size());

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                log!(logger, "Received a connection '{}'", stream.peer_addr()?);

                let logger_clone = logger.clone();
                thread_pool.enqueue(move || {
                    let _ = job(stream, logger_clone);
                });
            }

            Err(err) => {
                log!(logger, "*** An error occured: '{}'", err);
            }
        }
    }

    Ok(())
}
