use std::{sync::{Arc, Mutex}, collections::HashMap, fmt::Display};

pub type Am<T> = Arc<Mutex<T>>;
pub type FilePath = String;
pub type FileName = String;
pub type IndexMap = HashMap<FileName, FilePath>;

pub fn escape_iter<I: Iterator<Item = impl Display>>(iter: I) -> Vec<String>
{
    let mut v = vec![];

    for entry in iter {
        v.push(url_encode(entry))
    }

    v
}

pub fn url_encode<S: Display>(input: S) -> String {
    let mut encoded = String::new();
    for byte in input.to_string().bytes() {
        match byte {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'.' | b'_' | b'~' => {
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

pub fn url_decode<S: Display>(input: S) -> String {
    let mut decoded = String::new();
    let input_string = input.to_string();
    let mut bytes = input_string.bytes();
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
