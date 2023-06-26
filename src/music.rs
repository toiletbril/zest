use std::io::{Error, Write, Read, SeekFrom, Seek};
use std::net::TcpStream;
use std::fs::{File};

use crate::common::Am;
use crate::{log, Logger};
use crate::http::HttpConnection;

// Temporal file for testing :3c
// You can get this track at <https://www.youtube.com/watch?v=hqXDCTJFutY>
const FILE: &str = "01 Friday Night Clubbers Die By The Sword.mp3";

const CHUNK_SIZE: usize = 1024 * 1024;

pub fn serve_music_chunk(connection: &mut HttpConnection, logger: &Am<Logger>) -> Result<(), Error> {
    let mut file = File::open(&FILE)?;

    // TODO: This should not be hardcoded
    let chunk_index = 0;
    let start_pos = chunk_index * CHUNK_SIZE as usize;

    file.seek(SeekFrom::Start(start_pos as u64))?;

    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let bytes_read = match file.read(&mut buffer[..CHUNK_SIZE as usize]) {
        Ok(bytes_read) => bytes_read,
        Err(err) => {
            log!(logger, "*** Encountered an error while reading a chunk: {}.", err);
            connection.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n")?;
            return Err(err);
        }
    };

    log!(logger, "Serving a chunk '{}' [{}..{}].", FILE, start_pos, start_pos + CHUNK_SIZE);
    connection.write_all(b"HTTP/1.1 200 OK\r\n")?;
    connection.write_all(b"Content-Type: audio/mpeg\r\n")?;
    connection.write_all(format!("Content-Length: {}\r\n\r\n", bytes_read).as_bytes())?;
    connection.write_all(&buffer[..bytes_read])?;
    connection.flush()?;

    Ok(())
}

pub fn handle_stream(stream: TcpStream, logger: Am<Logger>) -> Result<(), Error> {
    let mut connection = HttpConnection::from(stream);
    log!(logger, "Handling {:?}", connection);

    serve_music_chunk(&mut connection, &logger)?;

    log!(logger, "Closing {:?}", connection.stream);
    drop(connection);

    Ok(())
}
