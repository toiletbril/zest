use std::{fs::File};
use std::io::{Error, Read, Seek, SeekFrom};

use crate::{common::util::{Am, escape_iter}};
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::{log, Logger, Log};

use super::index::get_music_index;

const CHUNK_SIZE: usize = 1024 * 512; // 512 KB

pub fn list_handler<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Error> {
    let index = get_music_index();
    let index_result = index.read();

    if let Ok(index) = index_result {
        log!(logger, "Responding with music list to {:?}", connection.stream());
        HttpResponse::new(200, "OK")
            .set_header("Content-Type", "application/json")
            .set_body(format!("{:?}", escape_iter(index.map().keys())).as_bytes())
            .send(connection)
    } else {
        unreachable!()
    }
}

pub fn chunk_handler<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Error> {
    let params = connection.params();

    let chunk: usize = params
        .and_then(|x| x.get("chunk"))
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(0);

    let track_result = params.and_then(|x| x.get("name"));

    if let Some(filename) = track_result {
        let index = get_music_index();
        let index_map = index.read();
        if let Ok(index) = index_map {
            let filepath = index.map().get(filename);
            if let Some(path) = filepath {
                return serve_music_chunk(connection, logger, chunk, format!("{}{}", index.path(), path))
            } else {
                return Ok(HttpResponse::new(404, "Not Found")
                            .set_header("Content-Type", "application/json")
                            .set_body("{ \"message\": \"Track specified was not found\" }".as_bytes())
                            .send(connection)?
                )
            }
        }
    }

    HttpResponse::new(400, "Bad Request")
        .set_header("Content-Type", "application/json")
        .set_body("{ \"message\": \"Please specify track and chunk with path parameters\" }".as_bytes())
        .send(connection)
}

fn serve_music_chunk<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
    chunk_index: usize,
    path: String,
) -> Result<(), Error> {
    let mut file = File::open(&path)?;
    let max_size = file.metadata().map(|x| x.len()).unwrap_or(0) as usize;

    let start_pos = chunk_index * CHUNK_SIZE;

    if max_size < start_pos {
        return HttpResponse::new(416, "Range Not Satisfiable")
                .set_header("Content-Type", "application/json")
                .set_body("{ \"message\": \"Chunk is out of bounds.\" }".as_bytes())
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

    log!(
        logger,
        "Serving a chunk '{}' [{}..{}] to {:?}.",
        path,
        start_pos,
        start_pos + CHUNK_SIZE,
        connection.stream()
    );

    HttpResponse::new(200, "OK")
        .set_header("Content-Type", "audio/mpeg")
        .set_header("Content-Length", bytes_read)
        .set_body(&buffer[..bytes_read])
        .send(connection)
}
