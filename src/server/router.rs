use std::error::Error;

use crate::{
    common::logger::Logger,
    common::util::Am,
    http::{connection::{HttpConnection, HttpMethod}, response::HttpResponse},
    music::endpoint::{chunk_handler, list_handler},
};

pub fn handle_routes<'a>(
    connection: &mut HttpConnection,
    logger: &Am<Logger>,
) -> Result<(), Box<dyn Error>> {
    let method_and_path = (connection.method(), connection.path().as_ref());

    let result = match method_and_path {
        (HttpMethod::GET, "/api/v1/music/get") => chunk_handler(connection, logger),
        (HttpMethod::GET, "/api/v1/music/all") => list_handler(connection, logger),
        _ => not_found().send(connection),
    }?;

    Ok(result)
}

fn not_found<'a>() -> HttpResponse<'a> {
    HttpResponse::new(404, "Not Found").set_json_body(&"{ \"message\": \"Page not found\" }")
}
