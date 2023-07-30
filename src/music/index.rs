use crate::common::{util::{FileName, FilePath, IndexMap}, logger::Verbosity};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write},
    path::Path,
    sync::Once,
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

    pub fn get(&self, item: &str) -> Option<FilePath> {
        if let Some(value) = self.map.get(item) {
            Some(self.path.clone() + value)
        } else {
            None
        }
    }

    pub fn path(&self) -> &FilePath {
        &self.path
    }
}

static mut STATIC_MUSIC_INDEX: Result<MusicIndex, String> = Err(String::new());
static INIT_MUSIC: Once = Once::new();

pub fn init_music_index(path: String) -> Result<(), String> {
    unsafe {
        INIT_MUSIC.call_once(move || {
            if STATIC_MUSIC_INDEX.is_err() {
                match load_index(path) {
                    Ok(index) => STATIC_MUSIC_INDEX = Ok(index),
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

pub fn get_music_index() -> &'static MusicIndex {
    unsafe { STATIC_MUSIC_INDEX.as_ref().unwrap() }
}

// {"path": "...", "entries": [ { "...": "..." }, ...] }
fn load_index(path: String) -> Result<MusicIndex, Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut index_path = String::new();
    let mut entries: HashMap<FileName, FilePath> = HashMap::new();

    let mut cursor_position = 0;

    let mut buffer = [0; 1];

    let mut read_key = false;
    let mut in_array = false;
    let mut key = String::new();
    let mut value = String::new();

    loop {
        let size = reader.read(&mut buffer)?;

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
            '\r' => {}
            '\n' => {}
            '\"' => {
                let mut buffer = vec![];
                reader.read_until(b'\"', &mut buffer)?;
                buffer.pop(); // remove last "

                let quoted = String::from_utf8(buffer).map_err(|err| {
                    let message =
                        format!("Invalid UTF-8 sequence at position {} ({})", cursor_position, err);
                    Error::new(ErrorKind::InvalidInput, message)
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
                let message = format!("Invalid character '{}' at position {}", cursor_position, buffer[0] as char);
                return Err(Error::new(ErrorKind::InvalidInput, message));
            }
        }

        if &key == "path" && !value.is_empty() {
            index_path = value.clone();
        }

        cursor_position += 1;
    }

    if index_path.is_empty() || entries.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "File is not a zest index",
        ));
    }

    Ok(MusicIndex {
        path: index_path,
        map: entries,
    })
}

pub fn make_index(path: &FilePath, verbosity: Verbosity) -> Result<String, Error> {
    make_index_file(recurse_music(&path, verbosity)?, path)
}

fn recurse_music(path: &String, verbosity: Verbosity) -> Result<HashMap<FileName, FilePath>, Error> {
    fn internal(path: &String, initial_path_len: usize, verbosity: &Verbosity) -> Result<HashMap<FileName, FilePath>, Error> {
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
                    verbosity.print_if(Verbosity::Details, format!("Adding {}...", filename));

                    filepath.drain(..initial_path_len);
                    index.insert(
                        filename.trim_end_matches(".mp3").to_owned(),
                        filepath.trim_start_matches(path.as_str()).to_owned(),
                    );
                }
            } else {
                verbosity.print_if(Verbosity::Details, format!("Entering {:?}...", file.file_name()));

                index.extend(internal(&filepath, initial_path_len, verbosity)?);
            }
        }

        Ok(index)
    }

    internal(path, path.len(), &verbosity)
}

fn make_index_file(index: HashMap<FileName, FilePath>, path: &FilePath) -> Result<String, Error> {
    if index.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Directory does not contain any of the supported music files"
        ));
    }

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
