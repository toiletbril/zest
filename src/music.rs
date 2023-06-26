use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::Am;
use crate::http::HttpConnection;
use crate::{log, Logger};

// Temporal file for testing :3c
// You can get this track at <https://www.youtube.com/watch?v=hqXDCTJFutY>
const FILE: &str = "01 Friday Night Clubbers Die By The Sword.mp3";

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

pub fn music_handler(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Box<dyn Error>> {
    if let Err(err) = serve_music_chunk(connection, logger, 0) {
        Err(Box::new(err))
    } else {
        Ok(())
    }
}

pub fn serve_music_chunk(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
    chunk_index: usize,
) -> Result<(), std::io::Error> {
    let mut file = File::open(&FILE)?;

    let start_pos = chunk_index * CHUNK_SIZE as usize;

    file.seek(SeekFrom::Start(start_pos as u64))?;

    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let bytes_read = match file.read(&mut buffer[..CHUNK_SIZE as usize]) {
        Ok(bytes_read) => bytes_read,
        Err(err) => {
            connection.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n")?;
            return Err(err);
        }
    };

    log!(
        logger,
        "Serving a chunk '{}' [{}..{}].",
        FILE,
        start_pos,
        start_pos + CHUNK_SIZE
    );

    connection.write_all(b"HTTP/1.1 200 OK\r\n")?;
    connection.write_all(b"Content-Type: audio/mpeg\r\n")?;
    connection.write_all(format!("Content-Length: {}\r\n\r\n", bytes_read).as_bytes())?;
    connection.write_all(&buffer[..bytes_read])?;
    connection.flush()?;

    Ok(())
}
