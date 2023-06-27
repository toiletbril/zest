use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};
use std::sync::{Arc, Once, RwLock};

use crate::common::{Am, MusicIndex};
use crate::http::connection::HttpConnection;
use crate::http::response::HttpResponse;
use crate::{log, Log, Logger};

use super::index::load_index;

const CHUNK_SIZE: usize = 1024 * 512; // 512 KB

static mut STATIC_MUSIC_INDEX: Result<Arc<RwLock<MusicIndex>>, String> =
    Err(String::new());
static INIT_MUSIC: Once = Once::new();

// TODO:
// Who doesn't love hacks
pub fn init_music_index(path: String) -> Result<(), String> {
    unsafe {
        INIT_MUSIC.call_once(move || {
            if STATIC_MUSIC_INDEX.is_err() {
                match load_index(path) {
                    Ok(index) => STATIC_MUSIC_INDEX = Ok(Arc::new(RwLock::new(index))),
                    Err(err) => {
                        STATIC_MUSIC_INDEX = Err(err.to_string())
                    }
                }
            }
        });
        match &STATIC_MUSIC_INDEX {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

pub unsafe fn get_music_index() -> Arc<RwLock<MusicIndex>> {
    unsafe { STATIC_MUSIC_INDEX.as_ref().unwrap().clone() }
}

pub fn list_handler<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Error> {
    let index = unsafe { get_music_index() };
    let index_map = index.read();

    if let Ok(index_map) = index_map {
        log!(logger, "Responding with music list to {:?}", connection.stream());
        HttpResponse::new(200, "OK")
            .set_header("Content-Type", "application/json")
            .set_body(format!("{:?}", index_map.keys()).as_bytes())
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

    let track_result = params.and_then(|x| x.get("track"));

    if let Some(filename) = track_result {
        let index = unsafe { get_music_index() };
        let index_map = index.read();
        if let Ok(index_map) = index_map {
            let filepath = index_map.get(filename);
            if let Some(path) = filepath {
                serve_music_chunk(connection, logger, chunk, path.to_string())?;
            } else {
                HttpResponse::new(404, "Not Found").send(connection)?;
            }
        }
    }
    HttpResponse::new(404, "Not Found").send(connection)
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
        return HttpResponse::new(416, "Range Not Satisfiable").send(connection);
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
