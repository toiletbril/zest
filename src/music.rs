use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};

use crate::common::Am;
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::{log, Logger, Log};

// Temporal file for testing :3c
// You can get this track at <https://www.youtube.com/watch?v=hqXDCTJFutY>
const FILE: &str = "01 Friday Night Clubbers Die By The Sword.mp3";

const CHUNK_SIZE: usize = 1024 * 512; // 512 KB

pub fn music_handler<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Error> {
    let chunk: usize = connection.params()
        .and_then(|x| x.get("chunk"))
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(0);

    serve_music_chunk(connection, logger, chunk)
}

pub fn serve_music_chunk<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
    chunk_index: usize,
) -> Result<(), Error> {
    let mut file = File::open(&FILE)?;
    let max_size = file.metadata()
        .map(|x| x.len())
        .unwrap_or(0) as usize;

    let start_pos = chunk_index * CHUNK_SIZE;

    if max_size < start_pos {
        return HttpResponse::new(416, "Range Not Satisfiable")
            .send(connection);
    }

    file.seek(SeekFrom::Start(start_pos as u64))?;

    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let bytes_read = match file.read(&mut buffer[..CHUNK_SIZE as usize]) {
        Ok(bytes_read) => bytes_read,
        Err(err) => {
            return Err(err);
        }
    };

    log!(logger,
        "Serving a chunk '{}' [{}..{}] to {:?}.", FILE, start_pos, start_pos + CHUNK_SIZE, connection.stream());

    HttpResponse::new(200, "OK")
        .set_header("Content-Type", "audio/mpeg")
        .set_header("Content-Length", bytes_read)
        .set_body(&buffer[..bytes_read])
        .send(connection)
}
