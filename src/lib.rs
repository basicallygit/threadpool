use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::io;

/// A pool of thread workers which can be told
/// to execute jobs
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new thread pool of size `size`
    ///
    /// # Errors
    ///
    /// This function will return an error if the `Builder::spawn`
    /// function returns an error (the OS failed to create a thread)
    ///
    /// # Panics
    /// 
    /// Panics if a `size` of value `0` is passed in.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let threadpool = ThreadPool::new(5).unwrap();
    ///
    /// threadpool.execute(move || {
    ///     // thread code
    /// });
    /// ```
    pub fn new(size: usize) -> io::Result<ThreadPool> {
        assert!(size > 0);
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver))?);
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Pushes a new job to the job queue to be executed once a
    /// worker is free.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let threadpool = ThreadPool::new(5).unwrap();
    ///
    /// threadpool.execute(move || {
    ///     // thread code
    /// });
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> io::Result<Worker> {
        let thread = thread::Builder::new().spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        })?;

        Ok(Worker {
            thread: Some(thread),
        })
    }
}
