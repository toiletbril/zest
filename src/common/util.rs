#![allow(dead_code)]

use std::{io::{Error, ErrorKind}, sync::{Arc, Mutex}, collections::HashMap, fmt::Display};

pub type Am<T> = Arc<Mutex<T>>;
pub type FilePath = String;
pub type FileName = String;
pub type IndexMap = HashMap<FileName, FilePath>;

pub fn iter_to_json_string<I: Iterator<Item = impl AsRef<str>>>(iter: I) -> String {
    let mut s = String::from("[");

    let mut peekable = iter.peekable();

    while let Some(entry) = peekable.next() {
        s += "\"";
        s += entry.as_ref();
        s += "\"";
        if peekable.peek().is_some() {
            s += ",";
        }
    }

    s += "]";

    s
}

pub fn url_encode<S: Display>(input: S) -> String {
    let mut encoded = String::new();
    for byte in input.to_string().bytes() {
        match byte {
            b'0'..=b'9' |
            b'A'..=b'Z' |
            b'a'..=b'z' |
            b'-' |
            b'.' |
            b'_' |
            b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }

    encoded
}

pub fn url_decode<S: AsRef<str>>(input: S) -> Result<String, Error> {
    let mut input = input.as_ref().bytes();
    let mut decoded = Vec::new();

    while let Some(byte) = input.next() {
        match byte {
            b'%' => {
                if let (Some(hex1), Some(hex2)) = (input.next(), input.next()) {
                    let hex_str = format!("{}{}", hex1 as char, hex2 as char);

                    u8::from_str_radix(&hex_str, 16)
                        .map(|value| decoded.push(value))
                        .map_err(|err| {
                            Error::new(ErrorKind::InvalidInput, err.to_string())
                        })?;
                } else {
                    let err = Error::new(ErrorKind::InvalidInput, "Invalid URL string");
                    return Err(err);
                }
            }
            _ => {
                decoded.push(byte);
            }
        }
    }

    String::from_utf8(decoded).map_err(|err| {
        Error::new(ErrorKind::InvalidInput, err.to_string())
    })
}

mod tests {
    #[test]
    fn url_encode_and_decode() {
        use super::*;

        let first = "dsiaHello222World!!!(*&&(#^@()()))";
        let first_e = url_encode(first.clone());
        let first_d = url_decode(first_e.clone());

        println!("Input: {}", first);
        println!("Encoded: {}", first_e);

        assert_eq!(first, first_d.unwrap());

        let second = ".юю.Всем.ээдшовф**kjlOKOHQъъэОШЩЗ0привет%:?*()";
        let second_e = url_encode(second.clone());
        let second_d = url_decode(second_e.clone());

        println!("Input: {}", second);
        println!("Encoded: {}", second_e);

        assert_eq!(second, second_d.unwrap());

    }
}
