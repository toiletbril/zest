use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{Builder, JoinHandle};

use crate::{common::logger::{Log, Logger}, DEFAULT_THREAD_COUNT};
use crate::common::util::Am;
use crate::log;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Am<Receiver<Job>>, logger: Am<Logger>) -> Self {
        let receiver_clone = receiver.clone();

        let builder = Builder::new().name("worker".to_string());

        let worker_loop = move || loop {
            let to_exec = match receiver_clone.lock() {
                Ok(queue) => queue.recv(),
                Err(err) => {
                    log!(logger, "Shutting down: {}", err);
                    break;
                }
            };

            if let Ok(job) = to_exec {
                job()
            }
        };

        match builder.spawn(move || worker_loop()) {
            Ok(thread) => Worker {
                id: id,
                handle: Some(thread),
            },
            Err(err) => panic!("*** An error occured while creating worker thread: {}", err),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
    logger: Am<Logger>,
    size: usize,
}

const MAX_THREAD_AMOUNT: usize = 1024 * 8;

impl ThreadPool {
    pub fn new(mut size: usize, logger: Am<Logger>) -> Self {
        if size <= 0 || size > MAX_THREAD_AMOUNT {
            log!(logger, "*** Thread pool size is invalid. Using default: {}", DEFAULT_THREAD_COUNT);

            size = 8;
        }

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = channel();
        let receiver_ref = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, receiver_ref.clone(), logger.clone()));
        }

        ThreadPool {
            workers: workers,
            sender: Some(sender),
            logger: logger,
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

        if let Some(sender) = &self.sender {
            match sender.send(job) {
                Ok(()) => {}
                Err(_val) => todo!(),
            }
        } else {
            panic!("*** Dropped thread pool cannot execute more jobs.");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        log!(self.logger, "Dropping thread pool...");
        drop(self.sender.take());

        for worker in &mut self.workers {
            log!(self.logger, "Dropping worker {}...", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
