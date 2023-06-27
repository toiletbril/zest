use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Error, Write, ErrorKind, Read, BufRead, BufWriter},
    path::Path
};
use crate::{common::{FileName, FilePath}};

pub fn load_index(path: FilePath) -> Result<HashMap<FileName, FilePath>, Error> {
    let mut index = HashMap::new();
    let file = File::open(&path);

    if let Ok(mut file) = file {
        let mut buf: Vec<u8> = vec![];
        let _ = file.read_to_end(&mut buf);
        let mut lines = buf.lines();

        while let Some(Ok(line)) = lines.next() {
            let mut chars = line.chars();
            let mut filename = String::new();
            let mut filepath = String::new();

            if chars.next() != Some('"') {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "File is not a Zest index",
                ));
            }

            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                filename.push(c.to_ascii_lowercase());
            }

            if chars.by_ref().skip(4).next() != Some('"') {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "File is not a Zest index",
                ));
            }

            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                filepath.push(c);
            }

            index.insert(filename, filepath);
        }

        if index.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "File is not a Zest index",
            ));
        }

        Ok(index)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Could not open '{}': {}", path, file.unwrap_err()),
        ))
    }
}

pub fn make_index(path: FilePath) -> Result<String, Error> {
    make_index_file(recurse_music(path)?)
}

fn recurse_music(path: String) -> Result<HashMap<FileName, FilePath>, Error> {
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
                index.insert(filename, filepath);
            }
        } else {
            index.extend(recurse_music(filepath)?);
        }
    }

    Ok(index)
}

fn slugify(input: String) -> String {
    let mut slug = String::new();
    for c in input.chars() {
        if c.is_ascii_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation() {
            slug.push(c.to_lowercase().next().unwrap());
        }
    }
    slug.trim().to_string()
}

fn make_index_file(index: HashMap<FileName, FilePath>) -> Result<String, Error> {
    let mut i = 0;

    while Path::new(format!("./zest-index-{}", i).as_str()).exists() {
        i += 1;
    }

    let file = File::create(format!("./zest-index-{}", i))?;
    let mut writer = BufWriter::new(file);

    let mut files = index.iter();
    while let Some((filename, filepath)) = files.next() {
        writer.write_all(b"\"")?;
        writer.write_all(slugify(filename.to_owned()).as_bytes())?;
        writer.write_all(b"\" => \"")?;
        writer.write_all(filepath.as_bytes())?;
        writer.write_all(b"\"\n")?;
    }

    Ok(format!("./zest-index-{}", i))
}
