use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{Builder, JoinHandle};

use crate::common::logger::{Log, Logger};
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

        let main_loop = move || loop {
            {
                let to_exec = match receiver_clone.lock() {
                    Ok(queue) => queue.recv().ok(),
                    Err(err) => {
                        log!(logger, "*** Shutting down: {}", err);
                        break;
                    }
                };

                if let Some(job) = to_exec {
                    job()
                }
            }
        };

        match builder.spawn(move || main_loop()) {
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

impl ThreadPool {
    pub fn new(size: usize, logger: Am<Logger>) -> Self {
        assert!(size > 0, "Size should be greater than zero");
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
            panic!("*** Dropped thread pool can not execute more jobs.");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        log!(self.logger, "*** Dropping thread pool...");
        drop(self.sender.take());

        for worker in &mut self.workers {
            log!(self.logger, "*** Dropping worker {}...", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
