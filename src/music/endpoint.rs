use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};

use crate::common::{logger::Verbosity, util::Am};
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::music::index::get_music_index;
use crate::{log, log_geq, Log, Logger};

const CHUNK_SIZE: usize = 1024 * 128; // 128 kb

pub fn list_handler(connection: &mut HttpConnection, logger: &Am<Logger>) -> Result<(), Error> {
    let index = get_music_index();

    log!(logger, "{} <= Music list", connection.peer_string());

    return HttpResponse::new(200, "OK")
        .allow_all_origins(connection)
        .set_json_body(&index.key_json_array())
        .send(connection);
}

pub fn chunk_handler(connection: &mut HttpConnection, logger: &Am<Logger>) -> Result<(), Error> {
    let params = connection.params();

    let chunk = params
        .and_then(|x| x.get("chunk"))
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(0 as usize);

    let track_result = params.and_then(|x| x.get("name"));

    if let Some(filename) = track_result {
        let filepath = get_music_index().get(filename);

        if let Some(path) = filepath {
            return serve_music_chunk(connection, logger, chunk, path);
        } else {
            log!(logger, "{} <= 404 No such track",
                 connection.peer_string());

            return Ok(HttpResponse::new(404, "Not Found")
                .set_json_body(&"{ \"message\": \"Track specified was not found\" }")
                .allow_all_origins(connection)
                .send(connection)?);
        }
    }

    log!(logger, "{} <= 400 No name parameter", connection.peer_string());

    HttpResponse::new(400, "Bad Request")
        .set_json_body(&"{ \"message\": \"Please specify track and chunk with path parameters\" }")
        .allow_all_origins(connection)
        .send(connection)
}

fn serve_music_chunk(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
    chunk_index: usize,
    path: String,
) -> Result<(), Error> {
    log_geq!(logger, Verbosity::Debug, "Reading from '{}'...", path);

    let mut file = File::open(&path)?;
    let max_size = file.metadata()
        .map(|x| x.len()).unwrap_or(0) as usize;

    let start_pos = chunk_index * CHUNK_SIZE;

    if max_size < start_pos {
        return HttpResponse::new(416, "Range Not Satisfiable")
            .set_json_body(&"{ \"message\": \"Chunk is out of bounds.\" }")
            .allow_all_origins(connection)
            .send(connection);
    }

    file.seek(SeekFrom::Start(start_pos as u64))?;

    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let bytes_read = file.read(&mut buffer[..CHUNK_SIZE as usize])?;

    log!(logger, "{} <= Chunk {}, {}..{}",
         connection.peer_string(), chunk_index, start_pos, start_pos + CHUNK_SIZE);

    return HttpResponse::new(200, "OK")
        .set_header("Content-Type", "audio/mpeg")
        .set_header("Content-Length", bytes_read)
        .set_body(&buffer[..bytes_read])
        .allow_all_origins(connection)
        .send(connection);
}
