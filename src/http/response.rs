use std::{
    collections::HashMap,
    fmt::Display,
    io::{Error, Write},
};

use super::connection::HttpConnection;

#[derive(Debug)]
pub struct HttpResponse<'a> {
    status: u16,
    status_message: String,
    headers: Option<HashMap<String, String>>,
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
        let mut headers = self.headers.take().unwrap_or(HashMap::new());

        headers.insert(key.to_string(), value.to_string());

        self.headers = Some(headers);
        self
    }

    pub fn set_body(mut self, body: &'a [u8]) -> Self {
        self.body = Some(body);
        self
    }

    pub fn send(mut self, connection: &mut HttpConnection) -> Result<(), Error> {
        connection.write_all(format!("HTTP/1.1 {} {}\r\n", self.status, self.status_message).as_bytes())?;

        if let Some(headers) = self.headers.take() {
            for (key, value) in headers.iter() {
                connection.write_all(format!("{}: {}\r\n", key, value).as_bytes())?;
            }
        }

        connection.write_all(b"\r\n")?;

        if let Some(body) = self.body {
            connection.write_all(body)?;
        }

        connection.flush()?;

        Ok(())
    }
}
