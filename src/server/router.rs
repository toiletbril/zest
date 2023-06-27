use std::{error::Error};

use crate::{
    common::Am,
    http::{
        connection::{HttpConnection, HttpMethod},
        response::HttpResponse,
    },
    common::logger::Logger,
    api::music::music_handler, Config,
};

pub fn route<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
    config: Config
) -> Result<(), Box<dyn Error>> {
    Ok(match (connection.path().as_str(), connection.method()) {
        ("/api/v1/music", HttpMethod::GET) => music_handler(connection, logger, config),
        _ => not_found().send(connection),
    }?)
}

fn not_found<'a>() -> HttpResponse<'a> {
    HttpResponse::new(404, "Not Found")
        .set_header("Content-Type", "application/json")
        .set_body("{{message: \"not found\"}}".as_bytes())
}
