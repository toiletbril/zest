use std::{
    fmt::Display,
    io::{Error, Write},
};

use super::connection::HttpConnection;

#[derive(Debug)]
pub struct HttpResponse<'a> {
    status: u16,
    status_message: String,
    headers: Option<String>,
    body: Option<&'a [u8]>,
}

impl<'a> HttpResponse<'a> {
    pub fn new<S: Display>(status: u16, status_message: S) -> Self {
        return Self {
            status: status,
            status_message: status_message.to_string(),
            headers: None,
            body: None,
        };
    }

    pub fn set_header<K: Display, V: Display>(mut self, key: K, value: V) -> Self {
        let mut headers = self.headers.take().unwrap_or_default();
        headers.push_str(format!("{}: {}\r\n", key, value).as_str());
        self.headers = Some(headers);
        self
    }

    pub fn set_body(mut self, body: &'a [u8]) -> Self {
        self.body = Some(body);
        self
    }

    pub fn set_json_body<S: AsRef<str>>(self, body: &'a S) -> Self {
        self.set_header("Content-Type", "application/json; charset=utf-8")
            .set_body(body.as_ref().as_bytes())
    }

    pub fn allow_all_origins(self, connection: &HttpConnection) -> Self {
        let host = connection.headers().get("origin");
        self.set_header("Access-Control-Allow-Origin", host.unwrap_or(&"*".to_string()))
    }

    pub fn send(mut self, connection: &mut HttpConnection) -> Result<(), Error> {
        connection.write_all(
            format!("HTTP/1.1 {} {}\r\n", self.status, self.status_message).as_bytes(),
        )?;

        if let Some(headers) = self.headers.take() {
            connection.write_all(headers.as_bytes())?;
        }

        connection.write_all(b"\r\n")?;

        if let Some(body) = self.body {
            connection.write_all(body)?;
        }

        connection.flush()?;

        Ok(())
    }
}
