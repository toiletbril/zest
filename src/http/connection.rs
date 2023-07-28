use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Unknown,
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::Unknown
    }
}

#[derive(Debug)]
pub enum HttpVersion {
    V0_9,
    V1,
    V1_1,
    Unknown
}

impl Default for HttpVersion {
    fn default() -> Self {
        HttpVersion::Unknown
    }
}

#[derive(Debug)]
pub struct HttpConnection {
    stream: TcpStream,
    method: HttpMethod,
    path: Path,
    raw_path: String,
    version: HttpVersion,
    headers: Headers,
    parameters: Option<Parameters>,
}

impl Write for HttpConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

use std::collections::HashMap;
use std::str;

use crate::common::util::url_decode;

type Headers = HashMap<String, String>;
type Parameters = HashMap<String, String>;
type Path = String;

const MAX_HEADER_SIZE: usize = 1024 * 4;

/// Returns `Err` when:
/// - size of headers exceeded `MAX_HEADER_SIZE`.
/// - the request line is malformed.
/// - a header is malformed.
fn parse_http_headers(
    connection: &mut HttpConnection,
) -> Result<(), Error> {
    let mut total_bytes_read = 0;
    let mut current_line = String::new();
    let mut prev_character: Option<char> = None;
    let mut buffer = [0; 1];

    loop {
        match connection.stream_mut().read(&mut buffer)? {
            0 => break,
            _ => {
                let character = buffer[0] as char;
                total_bytes_read += 1;

                if character == '\n' {
                    if prev_character == Some('\r') && current_line.is_empty() {
                        return Ok(());
                    } else if connection.path().is_empty() {
                        let (method, path, raw_path, parameters, version) = parse_request_line(&current_line)?;

                        connection.method = method;
                        connection.path = path;
                        connection.raw_path = raw_path;
                        connection.version = version;
                        connection.parameters = parameters;
                    } else {
                        parse_header_line(&current_line, &mut connection.headers)?;
                    }

                    current_line.clear();
                } else if character != '\r' {
                    current_line.push(character);
                }

                prev_character = Some(character);
            }
        }

        if total_bytes_read > MAX_HEADER_SIZE {
            let message = format!("Header size exceeded {} bytes.", MAX_HEADER_SIZE);
            let err = std::io::Error::new(ErrorKind::OutOfMemory, message);
            return Err(err);
        }
    }

    let message = format!("Malformed headers.");
    let err = std::io::Error::new(ErrorKind::InvalidData, message);
    return Err(err);
}

fn parse_request_line(line: &str) -> Result<(HttpMethod, Path, String, Option<Parameters>, HttpVersion), Error> {
    // This supports multiple spaces between values on request line.
    let mut parts = line.split_whitespace();

    if parts.clone().count() < 2 {
        let message = "Invalid request line.";
        let err = Error::new(ErrorKind::InvalidInput, message);
        return Err(err);
    }

    let method = match parts.next().unwrap() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "PATCH" => HttpMethod::PATCH,
        "DELETE" => HttpMethod::DELETE,
        _ => HttpMethod::Unknown,
    };

    let raw_path = parts.next().unwrap().to_owned();
    let mut path_split = raw_path.split('?');

    let path = path_split.next().unwrap_or("/").to_owned();
    let decoded_path = url_decode(path)?;

    let parameters = if let Some(raw_parameters) = path_split.next() {
        let mut parameters = HashMap::new();
        let split_parameters = raw_parameters.split('&');

        for param in split_parameters {
            let mut key_value = param.split('=');

            let key = key_value.next();

            if key == None || key.unwrap().is_empty() {
                continue;
            }

            let value = key_value.next().unwrap();
            let key = key.unwrap();

            let decoded_key = url_decode(key)?;
            let decoded_value = url_decode(value)?;

            parameters.insert(decoded_key.to_lowercase(), decoded_value);
        }

        Some(parameters)
    } else {
        None
    };

    let version: HttpVersion = if let Some(version_part) = parts.next() {
        match version_part.to_ascii_lowercase().as_ref() {
            "http/0.9" => HttpVersion::V0_9,
            "http/1.0" => HttpVersion::V1,
            "http/1.1" => HttpVersion::V1_1,
            _ => HttpVersion::Unknown
        }
    } else {
        HttpVersion::V1
    };

    let decoded_raw_path = url_decode(raw_path)?;

    Ok((method, decoded_path, decoded_raw_path, parameters, version))
}

fn parse_header_line(line: &str, headers: &mut HashMap<String, String>) -> Result<(), Error> {
    if let Some((key, value)) = line.split_once(':') {
        headers.insert(key.to_lowercase().to_owned(), value.trim().to_owned());
        Ok(())
    } else {
        let message = "Invalid header line.";
        let err = Error::new(ErrorKind::InvalidInput, message);
        Err(err)
    }
}

impl Drop for HttpConnection {
    fn drop(&mut self) {
        drop(&mut self.stream)
    }
}

#[allow(unused)]
impl HttpConnection {
    /// Parses and consumes TcpStream, making a HttpConnection out of it.
    ///
    /// Returns `Err` when:
    /// - size of headers exceeded `MAX_HEADER_SIZE`.
    /// - the request line is malformed.
    /// - a header is malformed.
    pub fn new(mut stream: TcpStream) -> Result<Self, Error> {
        let mut connection = HttpConnection {
            stream: stream,
            method: Default::default(),
            path: Default::default(),
            raw_path: Default::default(),
            version: Default::default(),
            headers: Default::default(),
            parameters: Default::default()
        };

        parse_http_headers(&mut connection)?;

        Ok(connection)
    }

    pub fn stream_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn peer_string(&self) -> String {
        if let Ok(ip) = self.stream().peer_addr() {
            ip.to_string()
        } else {
            "Unknown address".into()
        }
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn raw_path(&self) -> &String {
        &self.raw_path
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn params(&self) -> Option<&Parameters> {
        self.parameters.as_ref()
    }
}
