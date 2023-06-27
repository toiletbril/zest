use std::{error::Error};

use crate::{
    common::Am,
    http::{
        connection::{HttpConnection, HttpMethod},
        response::HttpResponse,
    },
    common::logger::Logger,
    music::endpoint::{list_handler, chunk_handler}
};

pub fn route<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Box<dyn Error>> {
    Ok(match (connection.path().as_str(), connection.method()) {
        ("/api/v1/music", HttpMethod::GET) => chunk_handler(connection, logger),
        ("/api/v1/music/list", HttpMethod::GET) => list_handler(connection, logger),
        _ => not_found().send(connection),
    }?)
}

fn not_found<'a>() -> HttpResponse<'a> {
    HttpResponse::new(404, "Not Found")
        .set_header("Content-Type", "application/json")
        .set_body("{ \"message\": \"Page not found\" }".as_bytes())
}
