use crate::common::util::{FileName, FilePath, IndexMap};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write},
    path::Path,
    sync::{Arc, Once, RwLock},
};

#[derive(Debug)]
pub struct MusicIndex {
    map: IndexMap,
    path: FilePath,
}

impl MusicIndex {
    pub fn map(&self) -> &IndexMap {
        &self.map
    }

    pub fn path(&self) -> &FilePath {
        &self.path
    }
}

static mut STATIC_MUSIC_INDEX: Result<Arc<RwLock<MusicIndex>>, String> = Err(String::new());
static INIT_MUSIC: Once = Once::new();

pub fn init_music_index(path: String) -> Result<(), String> {
    unsafe {
        INIT_MUSIC.call_once(move || {
            if STATIC_MUSIC_INDEX.is_err() {
                match load_index(path) {
                    Ok(index) => STATIC_MUSIC_INDEX = Ok(Arc::new(RwLock::new(index))),
                    Err(err) => STATIC_MUSIC_INDEX = Err(err.to_string()),
                }
            }
        });
        match &STATIC_MUSIC_INDEX {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

pub fn get_music_index() -> Arc<RwLock<MusicIndex>> {
    unsafe { STATIC_MUSIC_INDEX.as_ref().unwrap().clone() }
}

// {"path": "...", "entries": [ { "...": "..." }, ...] }
fn load_index(path: String) -> Result<MusicIndex, Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut index_path = String::new();
    let mut entries: HashMap<FileName, FilePath> = HashMap::new();

    let mut current_char_position = 0;

    let mut char_buf = vec![];
    let mut buffer = [0; 1];

    let mut read_key = false;
    let mut in_array = false;
    let mut key = String::new();
    let mut value = String::new();

    loop {
        let size = reader.read(&mut buffer)?;
        char_buf.extend(buffer);

        if size == 0 {
            break;
        }

        match buffer[0] as char {
            '{' => {}
            '}' => {}
            ':' => {}
            ',' => {}
            ' ' => {}
            '[' => {
                in_array = true;
                read_key = false;
            }
            ']' => {
                in_array = false;
            }
            '\n' => {}
            '\"' => {
                let mut buffer = vec![];
                reader.read_until(b'\"', &mut buffer)?;
                let _ = buffer.pop(); // remove last "

                let quoted = String::from_utf8(buffer).map_err(|err| {
                    let message =
                        format!("Invalid UTF-8 sequence at position {} ({})", current_char_position, err);
                    Error::new(ErrorKind::InvalidData, message)
                })?;

                if !read_key {
                    key = quoted;
                } else {
                    value = quoted;
                    if in_array {
                        entries.insert(key.clone(), value.clone());
                    }
                }
                read_key = !read_key;
            }
            _ => {
                let message = format!("Invalid character '{}' at position {}", current_char_position, buffer[0] as char);
                return Err(Error::new(ErrorKind::InvalidData, message));
            }
        }

        char_buf.clear();

        if &key == "path" && !value.is_empty() {
            index_path = value.clone();
        }

        current_char_position += 1;
    }

    if index_path.is_empty() || entries.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "File is not a zest index",
        ));
    }

    Ok(MusicIndex {
        path: index_path,
        map: entries,
    })
}

pub fn make_index(path: FilePath) -> Result<String, Error> {
    make_index_file(recurse_music(&path, None)?, path)
}

fn recurse_music(
    path: &String,
    path_len: Option<usize>,
) -> Result<HashMap<FileName, FilePath>, Error> {
    let mut dir = fs::read_dir(path.to_string())?;
    let mut index: HashMap<FileName, FilePath> = HashMap::new();

    while let Some(Ok(file)) = dir.next() {
        let mut filepath: String = file.path().to_string_lossy().into();
        let mut filename: String = file.file_name().to_string_lossy().into();

        if cfg!(target_os = "windows") {
            filepath = filepath.replace("\\", "/");
            filename = filename.replace("\\", "/");
        }

        if let Ok(true) = file.metadata().map(|x| x.is_file()) {
            if filepath.ends_with(".mp3") {
                filepath.drain(..path_len.unwrap_or(0));
                index.insert(
                    filename.trim_end_matches(".mp3").to_owned(),
                    filepath.trim_start_matches(path.as_str()).to_owned(),
                );
            }
        } else {
            if let Some(len) = path_len {
                index.extend(recurse_music(&filepath, Some(len))?);
            } else {
                index.extend(recurse_music(&filepath, Some(path.len()))?);
            }
        }
    }

    Ok(index)
}

fn make_index_file(index: HashMap<FileName, FilePath>, path: FilePath) -> Result<String, Error> {
    let mut i = 0;

    while Path::new(format!("./zest-index-{}.json", i).as_str()).exists() {
        i += 1;
    }

    let filename = format!("./zest-index-{}.json", i);

    let file = File::create(&filename)?;
    let mut writer = BufWriter::new(file);

    write!(writer, "{{\"path\":\"{path}\",\"entries\":[")?;

    let mut files = index.iter().peekable();
    while let Some((filename, filepath)) = files.next() {
        writer.write_all(b"{\"")?;
        writer.write_all(filename.to_owned().as_bytes())?;
        writer.write_all(b"\":\"")?;
        writer.write_all(filepath.as_bytes())?;
        writer.write_all(b"\"}")?;
        if &files.peek() != &None {
            writer.write_all(b",")?;
        }
    }

    write!(writer, "]}}")?;

    Ok(filename)
}
