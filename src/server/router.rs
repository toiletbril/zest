use std::error::Error;

use crate::{
    common::logger::Logger,
    common::util::Am,
    http::{
        connection::{HttpConnection, HttpMethod},
        response::HttpResponse,
    },
    music::endpoint::{chunk_handler, list_handler},
};

pub fn handle_routes<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Box<dyn Error>> {
    Ok(match (connection.path().as_str(), connection.method()) {
        ("/api/v1/music/get", HttpMethod::GET) => chunk_handler(connection, logger),
        ("/api/v1/music/all", HttpMethod::GET) => list_handler(connection, logger),
        _ => not_found().send(connection),
    }?)
}

fn not_found<'a>() -> HttpResponse<'a> {
    HttpResponse::new(404, "Not Found").set_json_body(&"{ \"message\": \"Page not found\" }")
}
