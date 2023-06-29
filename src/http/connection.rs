use std::{io::{Error, ErrorKind, Read, Write}};
use std::net::TcpStream;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    UNKN,
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::UNKN
    }
}

#[derive(Debug)]
pub struct HttpConnection {
    stream: TcpStream,
    method: HttpMethod,
    path: Path,
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
    stream: &mut TcpStream,
) -> Result<(Headers, Option<Parameters>, HttpMethod, Path), Error> {
    let mut headers = HashMap::new();
    let mut parameters = None;

    let mut method = HttpMethod::UNKN;
    let mut path = String::new();

    let mut total_bytes_read = 0;
    let mut current_line = String::new();
    let mut prev_character: Option<char> = None;
    let mut buffer = [0; 1];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let character = buffer[0] as char;
                total_bytes_read += 1;

                if character == '\n' {
                    if prev_character == Some('\r') && current_line.is_empty() {
                        return Ok((headers, parameters, method, path));
                    } else if path.is_empty() {
                        let parsed = parse_request_line(&current_line);
                        if let Ok((method_parsed, path_parsed, parameters_parsed)) = parsed {
                            method = method_parsed;
                            path = path_parsed;
                            parameters = parameters_parsed;
                        } else {
                            return Err(parsed.unwrap_err());
                        }
                    } else {
                        if let Err(err) = parse_header_line(&current_line, &mut headers) {
                            return Err(err);
                        }
                    }

                    current_line.clear();
                } else if character != '\r' {
                    current_line.push(character);
                }

                prev_character = Some(character);
            }
            Err(err) => return Err(err), // error reading from the stream
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

fn parse_request_line(line: &str) -> Result<(HttpMethod, Path, Option<Parameters>), Error> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 2 {
        let message = "Invalid request line.";
        let err = Error::new(ErrorKind::InvalidInput, message);
        return Err(err);
    }

    let method = match parts[0] {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "PATCH" => HttpMethod::PATCH,
        "DELETE" => HttpMethod::DELETE,
        _ => HttpMethod::UNKN,
    };

    let raw_path = parts[1].to_owned();
    let mut path_split = raw_path.split('?');

    let path = path_split.next().unwrap_or("/").to_owned();

    let parameters = if let Some(raw_parameters) = path_split.next() {
        let mut parameters = HashMap::new();
        let split_parameters = raw_parameters.split('&');

        for param in split_parameters {
            let mut kv = param.split('=');

            let k = kv.next();
            if k == None || k.unwrap().is_empty() {
                continue;
            }
            let v = kv.next().unwrap_or_default();

            parameters.insert(url_decode(k.unwrap()).to_lowercase(), url_decode(v));
        }

        Some(parameters)
    } else {
        None
    };

    Ok((method, path, parameters))
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
        let result = parse_http_headers(&mut stream);
        if let Ok((headers, parameters, method, path)) = result {
            Ok(HttpConnection {
                stream,
                method,
                path,
                headers,
                parameters,
            })
        } else {
            Err(result.unwrap_err())
        }
    }

    pub fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn method(&self) -> HttpMethod {
        self.method.to_owned()
    }

    pub fn path(&self) -> String {
        self.path.to_owned()
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn params(&self) -> Option<&Parameters> {
        self.parameters.as_ref()
    }
}
