use crate::common::util::url_decode;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::{TcpStream, Shutdown};
use std::str;

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

#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    V0_9,
    V1,
    V1_1,
    Unknown,
}

impl Default for HttpVersion {
    fn default() -> Self {
        HttpVersion::Unknown
    }
}

type Headers = HashMap<String, String>;
type Parameters = HashMap<String, String>;
type Path = String;

#[derive(Debug, Default)]
struct HttpRequest {
    method: HttpMethod,
    path: Path,
    raw_path: String,
    version: HttpVersion,
    headers: Headers,
    parameters: Option<Parameters>,
}

fn parse_request_line(line: &str, request: &mut HttpRequest) -> Result<(), Error> {
    // This supports multiple spaces between values on request line.
    let mut parts = line.split_whitespace();

    if parts.clone().count() < 2 {
        let message = "Invalid number of arguments in the request line";
        let err = Error::new(ErrorKind::InvalidInput, message);
        return Err(err);
    }

    let method = match parts.next().unwrap().to_lowercase().as_ref() {
        "get" => HttpMethod::GET,
        "post" => HttpMethod::POST,
        "put" => HttpMethod::PUT,
        "patch" => HttpMethod::PATCH,
        "delete" => HttpMethod::DELETE,
        _ => {
            let message = "Invalid method";
            let err = Error::new(ErrorKind::InvalidInput, message);
            return Err(err);
        }
    };

    let raw_path = parts.next().unwrap().to_owned();
    let decoded_raw_path = url_decode(&raw_path)?;

    let mut path_split = raw_path.split('?');

    let path = path_split.next().unwrap_or("/").to_owned();
    let decoded_path = url_decode(&path)?;

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
            "http/1.0" => HttpVersion::V1,
            "http/1.1" => HttpVersion::V1_1,
            _ => {
                let message = "Invalid HTTP version";
                let err = Error::new(ErrorKind::InvalidInput, message);
                return Err(err);
            }
        }
    } else {
        HttpVersion::V0_9
    };

    request.method = method;
    request.path = decoded_path;
    request.raw_path = decoded_raw_path;
    request.version = version;
    request.parameters = parameters;

    Ok(())
}

fn parse_header_line(line: &str, headers: &mut HashMap<String, String>) -> Result<(), Error> {
    if let Some((key, value)) = line.split_once(':') {

        let trimmed_key = key.trim().to_lowercase().to_owned();
        let trimmed_value = value.trim().to_owned();

        headers.insert(trimmed_key, trimmed_value);

        Ok(())
    } else {
        let message = "Invalid header line";
        let err = Error::new(ErrorKind::InvalidInput, message);
        Err(err)
    }
}

const MAX_HEADER_SIZE: usize = 1024 * 4;

/// Returns `Err` when:
/// - size of headers exceeded `MAX_HEADER_SIZE`.
/// - the request line is malformed.
/// - a header is malformed.
fn parse_http_request(stream: &mut TcpStream) -> Result<HttpRequest, Error> {
    let mut total_bytes_read = 0;
    let mut current_line = String::new();
    let mut prev_character: Option<char> = None;
    let mut buffer = [0; 1];

    let mut request: HttpRequest = Default::default();

    loop {
        match stream.read(&mut buffer)? {
            0 => break,
            _ => {
                let character = buffer[0] as char;
                total_bytes_read += 1;

                if character == '\n' {
                    if prev_character == Some('\r') && current_line.is_empty() {
                        return Ok(request);
                    } else if request.path.is_empty() {
                        parse_request_line(&current_line, &mut request)?;
                    } else {
                        parse_header_line(&current_line, &mut request.headers)?;
                    }

                    current_line.clear();
                } else if character != '\r' {
                    current_line.push(character);
                }

                prev_character = Some(character);
            }
        }

        if total_bytes_read > MAX_HEADER_SIZE {
            let message = format!("Header size exceeded {} bytes", MAX_HEADER_SIZE);
            let err = std::io::Error::new(ErrorKind::OutOfMemory, message);
            return Err(err);
        }
    }

    let message = format!("Malformed headers");
    let err = std::io::Error::new(ErrorKind::InvalidData, message);
    return Err(err);
}

#[derive(Debug)]
pub struct HttpConnection {
    stream: TcpStream,
    request: HttpRequest,
}

impl Write for HttpConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl Drop for HttpConnection {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
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
        let request = parse_http_request(&mut stream)?;

        let mut connection = HttpConnection {
            stream: stream,
            request: request,
        };

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
        self.request.method
    }

    pub fn path(&self) -> &String {
        &self.request.path
    }

    pub fn raw_path(&self) -> &String {
        &self.request.raw_path
    }

    pub fn headers(&self) -> &Headers {
        &self.request.headers
    }

    pub fn params(&self) -> Option<&Parameters> {
        self.request.parameters.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_line_http0_9() {
        let mut request = HttpRequest::default();
        parse_request_line("GET /hello", &mut request).unwrap();

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/hello");
        assert_eq!(request.version, HttpVersion::V0_9);
    }

    #[test]
    fn test_parse_request_line_http1_0() {
        let mut request = HttpRequest::default();
        parse_request_line("POST /world HTTP/1.0", &mut request).unwrap();

        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(request.path, "/world");
        assert_eq!(request.version, HttpVersion::V1);
    }

    #[test]
    fn test_parse_request_line_http1_1() {
        let mut request = HttpRequest::default();
        parse_request_line("DELETE /sailor HTTP/1.1", &mut request).unwrap();

        assert_eq!(request.method, HttpMethod::DELETE);
        assert_eq!(request.path, "/sailor");
        assert_eq!(request.version, HttpVersion::V1_1);
    }

    #[test]
    fn test_parse_request_line_unknown_method() {
        let mut request = HttpRequest::default();
        let result = parse_request_line("FOO /path HTTP/1.1", &mut request);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_request_line_unknown_version() {
        let mut request = HttpRequest::default();
        let result = parse_request_line("GET /path HTTP/69.0", &mut request);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_request_line_http1_0_extra_spaces() {
        let mut request = HttpRequest::default();
        parse_request_line("GET   /path   HTTP/1.0", &mut request).unwrap();

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/path");
        assert_eq!(request.version, HttpVersion::V1);
    }

    #[test]
    fn test_parse_request_line_http1_1_lower_case() {
        let mut request = HttpRequest::default();
        parse_request_line("get /path http/1.1", &mut request).unwrap();

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/path");
        assert_eq!(request.version, HttpVersion::V1_1);
    }

    #[test]
    fn test_parse_header_line_valid() {
        let mut headers = Headers::new();
        parse_header_line("Content-Type: text/html", &mut headers).unwrap();

        assert_eq!(headers.get("content-type").unwrap(), "text/html");
    }

    #[test]
    fn test_parse_header_line_invalid() {
        let mut headers = Headers::new();
        let result = parse_header_line("Invalid-Header", &mut headers);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_line_valid_extra_spaces() {
        let mut headers = Headers::new();
        parse_header_line("Content-Type   :   text/html", &mut headers).unwrap();

        assert_eq!(headers.get("content-type").unwrap(), "text/html");
    }

    #[test]
    fn test_parse_header_line_valid_case_insensitive() {
        let mut headers = Headers::new();
        parse_header_line("content-type: text/html", &mut headers).unwrap();

        assert_eq!(headers.get("content-type").unwrap(), "text/html");
    }

    fn start_test_listener(payload: &[u8]) -> TcpStream {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let mut writer = std::net::TcpStream::connect(addr).unwrap();

        writer.write(payload).unwrap();

        let (reader, _) = listener.accept().unwrap();

        reader
    }

    #[test]
    fn test_parse_http_request() {
        let payload =
            b"PATCH /api/v1/music/all?hello=world&what=nothing HTTP/1.0\r\nHost: www.example.com\r\n\r\n";

        let mut stream = start_test_listener(payload);

        match parse_http_request(&mut stream) {
            Ok(request) => {
                assert_eq!(request.method, HttpMethod::PATCH);
                assert_eq!(request.path, "/api/v1/music/all");
                assert_eq!(request.version, HttpVersion::V1);
                assert_eq!(request.headers.get("host").unwrap(), "www.example.com");
                assert_eq!(request.parameters.as_ref().unwrap().get("hello").unwrap(), "world");
                assert_eq!(request.parameters.as_ref().unwrap().get("what").unwrap(), "nothing");
            }
            Err(e) => panic!("Test failed: {:?}", e),
        }
    }
}
