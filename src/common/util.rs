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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{url_encode, url_decode, iter_to_json_string};

    fn rand(max: u32) -> usize {
        let start = SystemTime::now();

        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        (since_the_epoch.subsec_nanos() % max) as usize
    }

    fn generate_random_string() -> String {
        let ascii_characters = 0x0020..0x007F;
        let cyrillic_letters = 0x0400..0x04FF;
        let currency_symbols = 0x20A0..0x20CF;

        let characters: Vec<char> = ascii_characters
            .chain(cyrillic_letters)
            .chain(currency_symbols)
            .filter_map(char::from_u32)
            .collect();

        let char_table_len = characters.len() as u32;
        let string_len = rand(100) + 1;

        let s: String = (0..string_len)
            .map(|_| characters[rand(char_table_len)])
            .collect();
        s
    }

    #[test]
    fn url_encode_and_decode() {
        for _ in 0..1000 {
            let original = generate_random_string();
            let encoded = url_encode(&original);
            let decoded = url_decode(&encoded).unwrap();

            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn make_json_from_iter() {
        let iter = vec!["Hello", "World"].into_iter();

        assert_eq!(iter_to_json_string(iter), "[\"Hello\",\"World\"]");
    }
}
