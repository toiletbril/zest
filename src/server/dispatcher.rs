use std::error::Error;
use std::net::{TcpListener, TcpStream};

use crate::common::logger::{Log, Logger, Verbosity};
use crate::common::threads::ThreadPool;
use crate::common::util::Am;
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::{log, log_geq, log_eq, log_leq};

type DispatcherJob = fn(&mut HttpConnection, &Am<Logger>) -> Result<(), Box<dyn Error>>;

/// Starts the dispatcher, creating `ThreadPool` with N threads to handle incoming connections.
/// Job is the function to execute on incoming connections.
///
/// Before returning `Ok`, jobs should send their own response with `HttpConnection`.
/// On `Err`, HTTP Code 500 is sent.
pub fn start_dispatcher<'a>(
    address: String,
    thread_count: usize,
    logger: &Am<Logger>,
    job: DispatcherJob,
) -> Result<(), std::io::Error> {
    log!(logger, "Binding to <http://{address}>...");

    let listener = TcpListener::bind(address)?;

    let thread_pool = ThreadPool::new(thread_count, logger.clone());

    log!(logger, "Started. Available threads: {}.", thread_pool.size());

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                log_geq!(logger, Verbosity::Debug, "Dispatching a thread for {:?}", stream);

                let logger_clone = logger.clone();

                thread_pool.enqueue(move || {
                    let _ = handle_stream(stream, logger_clone, job);
                });
            }
            Err(err) => {
                log!(logger, "*** An error has occured while receiving stream: {}", err);
            }
        }
    }

    Ok(())
}

fn handle_stream<'a>(
    stream: TcpStream,
    logger: Am<Logger>,
    job: DispatcherJob,
) -> Result<(), Box<dyn Error>> {
    let connection = HttpConnection::new(stream);

    if let Ok(mut connection) = connection {
        log_leq!(logger, Verbosity::Default, "{} => {:?} {:?}",
            connection.peer_string(), connection.method(), connection.raw_path());

        log_eq!(logger, Verbosity::Details, "Connection: {:?}", connection.stream());

        log_geq!(logger, Verbosity::Debug, "Connection: {:?}", connection);

        if let Err(err) = job(&mut connection, &logger) {
            HttpResponse::new(500, "Internal Server Error")
                .allow_all_origins(&connection)
                .send(&mut connection)?;

            log!(logger, "*** An internal error has occured: {}", err);

            Err(err)
        } else {
            log_geq!(logger, Verbosity::Debug, "Closing {:?}", connection.stream_mut());

            drop(connection);
            Ok(())
        }
    } else {
        let err = connection.unwrap_err();

        log!(logger, "*** An error has occured while parsing connection: {}", err);

        Err(Box::new(err))
    }
}
