use std::{error::Error, io::Write};

use crate::{
    common::Am,
    http::{HttpConnection, HttpMethod},
    logger::Logger,
    music::music_handler,
};

pub fn route(connection: &mut HttpConnection, logger: &Am<Logger>) -> Result<(), Box<dyn Error>> {
    match (connection.path().as_str(), connection.method()) {
        ("/music", HttpMethod::GET) => music_handler(connection, logger),
        _ => not_found(connection),
    }
}

fn not_found(connection: &mut HttpConnection) -> Result<(), Box<dyn Error>> {
    connection.write_all(b"HTTP/1.1 404 Not Found\r\n")?;
    connection.flush()?;
    Ok(())
}
