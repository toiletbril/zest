use std::{error::Error, io::Write};

use crate::{logger::Logger, http::HttpConnection, common::Am, music::music_handler};

pub fn route(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Box<dyn Error>> {
    let mut root = connection.path().split('/').skip(1);
    match root.next() {
        Some("music") => music_handler(connection, logger),
        _ => not_found(connection)
    }
}

fn not_found(
    connection: &mut HttpConnection,
) -> Result<(), Box<dyn Error>> {
    connection.write_all(b"HTTP/1.1 404 Not Found\r\n")?;
    connection.flush()?;
    Ok(())
}