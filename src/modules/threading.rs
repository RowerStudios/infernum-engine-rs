use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _id: usize,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let job = {
                let lock = receiver.lock().unwrap();
                lock.recv()
            };
            match job {
                Ok(job) => job(),
                Err(_) => break,
            }
        });
        Self {
            _id: id,
            handle: Some(handle),
        }
    }
}

// simple fixed-size thread pool for background work
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let size = size.max(1);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();
        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Box::new(job));
        }
    }

    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take();
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
}
