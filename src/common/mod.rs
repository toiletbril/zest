pub mod logger;
pub mod threads;

use std::{sync::{Arc, Mutex}, collections::HashMap};

pub type Am<T> = Arc<Mutex<T>>;
pub type FilePath = String;
pub type FileName = String;
pub type MusicIndex = HashMap<FileName, FilePath>;