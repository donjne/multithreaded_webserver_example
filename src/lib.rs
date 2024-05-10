use std::{sync::{Arc, Mutex, mpsc}, thread, error::Error};

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move|| loop {
            let message = receiver.lock().expect("Acquiring the lock failed").recv();
            match message {
                Ok(job) => {

                 println!("Worker {id} got a job while executing"); 
                 job() }
                 Err(_) => {
                    println!("Worker {id} disconnected. Shutting down.");
                    break
                 }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl Drop for ThreadPool{
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Failed to join worker thread"); 
            }
        }
    }
}

///Create an implementation of our Threadpool type
impl ThreadPool {
    /// Create a function that takes usize as the type of it's paramter
    /// The size is the number of threads you can create in the pool
    /// # Panics
    /// It panics if the size is less than zero
    pub fn new(size: usize) -> Result<ThreadPool, Box<dyn Error>> {
        if size == 0 {
            return Err("Threadpool size is insufficient".into());
        }
        let mut workers =Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Ok(ThreadPool { workers, sender: Some(sender) })
    }

/// Defining a method that executes the thread in the pool and calls a closure
/// Using the where clause to implement FnOnce which takes a closure
/// Send which allows us to transfer the closure from one thread to another and 
/// static which makes sure our ThreadPool is still in scope
    pub fn execute<F>(&self, f:F) -> Result<(), Box<dyn Error>> where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        if let Some(sender) = &self.sender {
            sender.send(job)?;
        } else {
            return Err("Threadpool is shutting down".into())
        }
        Ok(())
    }
    }