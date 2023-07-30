use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::util::{url_encode, url_decode};

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
