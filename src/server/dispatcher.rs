use std::error::Error;
use std::net::{TcpListener, TcpStream};

use crate::{common::logger::{Log, Logger, Verbosity}, log_matching_verbosity};
use crate::common::threads::ThreadPool;
use crate::common::util::Am;
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::log;

type DispatcherJob = fn(&mut HttpConnection, &Am<Logger>) -> Result<(), Box<dyn Error>>;

/// Starts the dispatcher, creating `ThreadPool` with N threads to handle incoming connections.
/// Job is the function to execute on incoming connections.
///
/// Before returning `Ok`, jobs should send their own response with `HttpConnection`.
/// On `Err`, HTTP Code 500 is sent.
pub fn start_dispatcher<'a>(
    address: String,
    thread_count: usize,
    logger: Am<Logger>,
    job: DispatcherJob,
) -> Result<(), std::io::Error> {
    log!(logger, Verbosity::Default, "Binding to <http://{address}>...");
    let listener = TcpListener::bind(address)?;
    let thread_pool = ThreadPool::new(thread_count, logger.clone());

    log!(logger, Verbosity::Default,
        "Started. Available threads: {}.", thread_pool.size());

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                log_matching_verbosity!(logger, Verbosity::Debug, "Received a connection '{:?}'", stream);

                let logger_clone = logger.clone();

                thread_pool.enqueue(move || {
                    let _ = handle_stream(stream, logger_clone, job);
                });
            }
            Err(err) => {
                log!(logger, Verbosity::Default,
                    "*** An error has occured while receiving stream: {}", err);
            }
        }
    }

    Ok(())
}

/// This handles TcpStreams.
/// It parses the stream and makes `HttpConnection` out of it, breaking the chain and logging an error if it failed.
fn handle_stream<'a>(
    stream: TcpStream,
    logger: Am<Logger>,
    job: DispatcherJob,
) -> Result<(), Box<dyn Error>> {
    let connection = HttpConnection::new(stream);

    if let Ok(mut connection) = connection {
        log!(logger, Verbosity::Default, "Handling {:?} {:?}", connection.method(), connection.path());
        log_matching_verbosity!(logger, Verbosity::Debug, "Connection debug details {:?}", connection);

        if let Err(err) = job(&mut connection, &logger) {
            HttpResponse::new(500, "Internal Server Error")
                .allow_all_origins(&connection)
                .send(&mut connection)?;

            log!(logger, Verbosity::Default, "*** An internal error has occured: {}", err);

            Err(err)
        } else {
            log!(logger, Verbosity::Debug, "Closing {:?}", connection.stream());

            drop(connection);
            Ok(())
        }
    } else {
        let err = connection.unwrap_err();

        log!(logger, Verbosity::Default,
            "*** An error has occured while parsing connection: {}", err);

        Err(Box::new(err))
    }
}
