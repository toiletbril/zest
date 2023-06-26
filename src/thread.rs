use std::sync::{Mutex, Arc};
use std::thread::{JoinHandle, Builder};
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::common::Am;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _thread: JoinHandle<()>,
    _receiver: Am<Receiver<Job>>,
}

impl Worker {
    fn new(_id: usize, receiver: Am<Receiver<Job>>) -> Self {
        let receiver_clone = receiver.clone();

        let builder = Builder::new()
            .name("worker".to_string());

        match builder.spawn(move || loop {
            let mut to_exec = None;

            if let Ok(queue) = receiver_clone.lock() {
                if let Ok(job) = queue.recv() {
                    to_exec = Some(job)
                }
            }

            if let Some(job) = to_exec {
                job()

            }
        }) {
            Ok(thread) => Worker {
                _thread: thread,
                _receiver: receiver,
            },
            Err(err) => panic!("*** An error occured while creating worker thread: {}", err)
        }
    }
}

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: Sender<Job>,
    size: usize,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Size should be greater than zero");
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = channel();
        let receiver_ref = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, receiver_ref.clone()));
        }

        ThreadPool {
            _workers: workers,
            sender: sender,
            size: size,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn enqueue<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(func);
        match self.sender.send(job) {
            Ok(()) => {}
            Err(_val) => todo!()
        }
    }
}