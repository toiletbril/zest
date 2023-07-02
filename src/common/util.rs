#![allow(dead_code)]

use std::{sync::{Arc, Mutex}, collections::HashMap, fmt::Display};

pub type Am<T> = Arc<Mutex<T>>;
pub type FilePath = String;
pub type FileName = String;
pub type IndexMap = HashMap<FileName, FilePath>;

pub fn make_json_array_string<I: Iterator<Item = impl AsRef<str>>>(iter: I) -> String
{
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

pub fn url_decode<S: AsRef<str>>(input: S) -> String {
    let input_string = input.as_ref();
    let mut bytes = input_string.bytes();

    let mut decoded = String::new();

    while let Some(byte) = bytes.next() {
        match byte {
            b'%' => {
                if let (Some(hex1), Some(hex2)) = (bytes.next(), bytes.next()) {
                    if let Ok(decoded_byte) = u8::from_str_radix(&format!("{}{}", hex1 as char, hex2 as char), 16) {
                        decoded.push(decoded_byte as char);
                    } else {
                        decoded.push('%');
                        decoded.push(hex1 as char);
                        decoded.push(hex2 as char);
                    }
                } else {
                    decoded.push('%');
                }
            }
            _ => {
                decoded.push(byte as char);
            }
        }
    }

    decoded
}
