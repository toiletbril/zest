use std::{sync::{Arc, Mutex}, collections::HashMap, fmt::Display};

use crate::http::connection::url_encode;

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
