use std::error::Error;
use std::net::{TcpListener, TcpStream};

use crate::common::Am;
use crate::http::HttpConnection;
use crate::log;
use crate::logger::Logger;
use crate::thread::ThreadPool;

pub fn start_dispatcher(
    address: String,
    thread_count: usize,
    logger: Am<Logger>,
    job: fn(&mut HttpConnection, &Am<Logger>) -> Result<(), Box<dyn Error>>,
) -> Result<(), std::io::Error> {
    log!(logger, "Binding to <http://{address}>...");

    let listener = TcpListener::bind(address)?;
    let thread_pool = ThreadPool::new(thread_count, logger.clone());

    log!(logger, "Started. Available threads: {}.", thread_pool.size());

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                log!(logger, "Received a connection '{}'", stream.peer_addr()?);

                let logger_clone = logger.clone();

                thread_pool.enqueue(move || {
                    let _ =  handle_stream(stream, logger_clone, job);
                });
            }

            Err(err) => {
                log!(logger, "*** An error has occured: '{}'", err);
            }
        }
    }

    Ok(())
}

fn handle_stream(
    stream: TcpStream,
    logger: Am<Logger>,
    job: fn(&mut HttpConnection, &Am<Logger>) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let connection = HttpConnection::new(stream);

    if let Ok(mut connection) = connection {
        log!(logger, "Handling {:?}", connection);

        if let Err(err) = job(&mut connection, &logger) {
            log!(logger, "*** An internal error has occured: {:?}", err);
            return Err(err);
        };

        log!(logger, "Closing {:?}", connection.stream());
        drop(connection);

        Ok(())
    } else {
        let err = connection.unwrap_err();
        log!(logger, "*** An error has occured while parsing connection: {:?}", err);
        Err(Box::new(err))
    }
}
