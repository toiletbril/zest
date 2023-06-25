use std::sync::{Arc, Mutex};

pub type Am<T> = Arc<Mutex<T>>;