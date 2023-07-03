use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};

use crate::common::util::{make_json_array_string, Am};
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::{log, Log, Logger};

use super::index::get_music_index;

const CHUNK_SIZE: usize = 1024 * 128; // 128 kb

pub fn list_handler<'a>(connection: &mut HttpConnection, logger: &Am<Logger>) -> Result<(), Error> {
    let index = get_music_index();
        log!(logger, "Responding with music list to {:?}",
            connection.stream());

        return HttpResponse::new(200, "OK")
            .allow_all_origins(connection)
            .set_json_body(&make_json_array_string(index.map().keys()))
            .send(connection);
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
        let filepath = index.map().get(filename);

        if let Some(path) = filepath {
            return serve_music_chunk(connection, logger, chunk,
                                    format!("{}{}", index.path(), path));
        } else {
            return Ok(HttpResponse::new(404, "Not Found")
                .set_json_body(&"{ \"message\": \"Track specified was not found\" }")
                .allow_all_origins(connection)
                .send(connection)?);
        }
    }

    HttpResponse::new(400, "Bad Request")
        .set_json_body(&"{ \"message\": \"Please specify track and chunk with path parameters\" }")
        .allow_all_origins(connection)
        .send(connection)
}

fn serve_music_chunk<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
    chunk_index: usize,
    path: String,
) -> Result<(), Error> {
    log!(logger, "Reading from '{}'...", path);

    let mut file = File::open(&path)?;
    let max_size = file.metadata().map(|x| x.len()).unwrap_or(0) as usize;

    let start_pos = chunk_index * CHUNK_SIZE;

    if max_size < start_pos {
        return HttpResponse::new(416, "Range Not Satisfiable")
            .set_json_body(&"{ \"message\": \"Chunk is out of bounds.\" }")
            .allow_all_origins(connection)
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

    log!(logger, "Serving a chunk '{}' [{}..{}] to {:?}.",
         path, start_pos, start_pos + CHUNK_SIZE, connection.stream());

    return HttpResponse::new(200, "OK")
        .set_header("Content-Type", "audio/mpeg")
        .set_header("Content-Length", bytes_read)
        .set_body(&buffer[..bytes_read])
        .allow_all_origins(connection)
        .send(connection);
}
