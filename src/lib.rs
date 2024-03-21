use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread::{self, JoinHandle}};

/// A thread pool that can execute jobs.
/// 
/// ## Fields
/// - `workers`: The workers in the pool.
/// - `sender`: The sender end of the channel. Used to send work to the workers.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

/// A job that can be executed by a worker.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// ## Parameters
    /// - `size`: The number of threads in the pool.
    /// 
    /// ## Returns
    /// A `ThreadPool` with `size` number of threads.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // Create a channel with a capacity of `size`.
        let (sender, receiver) = mpsc::channel();
        
        // Wrap the receiver in an `Arc` and a `Mutex` to make it thread safe.
        let receiver = Arc::new(Mutex::new(receiver));

        // Create a vector to hold the threads.
        let mut workers = Vec::with_capacity(size);

        // Create `size` threads and store them in the vector.
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        // Return the ThreadPool.
        ThreadPool { 
            workers, 
            sender: Some(sender) 
        }
    }

    /// Execute a job on the ThreadPool.
    /// 
    /// ## Parameters
    /// - `f`: The job to execute. This must implement `FnOnce()`.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

/// Implement the `Drop` trait for `ThreadPool`.
impl Drop for ThreadPool {
    /// Shut down the ThreadPool.
    fn drop(&mut self) {
        // Drop the sender to signal to the workers that they should shut down.
        drop(self.sender.take());
        
        // Loop through each worker and shut them down.
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // Send a message to each worker to tell them to shut down.
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// A worker that executes jobs.
/// 
/// ## Fields
/// - `id`: The id of the worker.
/// - `thread`: The thread that the worker is running on.
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    /// 
    /// ## Parameters
    /// - `id`: The id of the worker.
    /// - `receiver`: The receiver end of the channel.
    /// 
    /// ## Returns
    /// A new `Worker` with the given id and receiver.
    /// 
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        // Create a new thread that will run the worker's main loop.
        let thread = thread::spawn(move || loop {
            // Receive a job from the channel. If there are no more jobs, this call will block.
            let message = receiver.lock().unwrap().recv();

            // If the message is an error, the channel has been closed and the worker should shut down.
            match message {
                // If the message is Ok, execute the job.
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                },
                // If the message is an error, the channel has been closed and the worker should shut down.
                Err(_) => {
                    println!("Worker {} is shutting down.", id);
                    break;
                },
            }
        });

        Worker { 
            id, 
            thread: Some(thread), 
        }
    }
}

