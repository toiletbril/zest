use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    UNKN,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::GET
    }
}

#[derive(Debug)]
pub struct HttpConnection {
    stream: TcpStream,
    method: HttpMethod,
    path: Path,
    headers: Headers,
    parameters: Parameters,
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

type Headers = HashMap<String, String>;
type Parameters = HashMap<String, String>;
type Path = String;

fn parse_http_request(stream: &TcpStream) -> Option<(Parameters, Headers, HttpMethod, Path)> {
    let mut reader = BufReader::new(stream);

    let mut request_line = String::new();
    let _ = reader.read_line(&mut request_line);

    let lines: Vec<String> = reader.lines().filter_map(Result::ok).take_while(|x| x.len() > 2).collect();

    println!("{:#?}", lines);

    if lines.is_empty() {
        return None;
    }

    let request_line = &request_line[..request_line.len() - 2];

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();

    let mut parameters = HashMap::new();
    let mut headers = HashMap::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ':').map(|part| part.trim()).collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].to_string();
        let value = parts[1].to_string();

        if key.eq_ignore_ascii_case("Host") {
            if let Some(index) = value.find(':') {
                let host = value[..index].to_string();
                let port = value[(index + 1)..].to_string();
                headers.insert("Host".to_string(), host);
                headers.insert("Port".to_string(), port);
            } else {
                headers.insert("Host".to_string(), value);
            }
        } else {
            headers.insert(key, value);
        }
    }

    if let Some(index) = path.find('?') {
        let parameter_string = &path[(index + 1)..];
        let parameter_vec: Vec<&str> = parameter_string.split('&').collect();

        for parameter in parameter_vec {
            if let Some(index) = parameter.find('=') {
                let key = &parameter[..index];
                let value = &parameter[(index + 1)..];
                parameters.insert(key.to_string(), value.to_string());
            }
        }
    }

    let method = match method.as_str() {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "PATCH" => HttpMethod::PATCH,
        "DELETE" => HttpMethod::DELETE,
        _ => HttpMethod::UNKN,
    };

    let path = path.split('?').next().unwrap_or("/").to_string();

    Some((parameters, headers, method, path))
}

impl Drop for HttpConnection {
    fn drop(&mut self) {
        drop(&mut self.stream)
    }
}

impl From<TcpStream> for HttpConnection {
    fn from(stream: TcpStream) -> Self {
        if let Some((parameters, headers, method, path)) = parse_http_request(&stream) {
            HttpConnection {
                stream,
                method,
                path,
                headers,
                parameters,
            }
        } else {
            HttpConnection {
                stream,
                method: HttpMethod::UNKN,
                path: Path::new(),
                headers: Headers::new(),
                parameters: Parameters::new(),
            }
        }
    }
}
