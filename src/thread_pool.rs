use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;


/// Multiple worker threads
pub struct ThreadPool
{
	threads: Vec<Thread>,
	sender: Option<Sender<Job>>,
}


/// An execution thread
struct Thread
{
	thread: Option<JoinHandle<()>>,
}


/// A function to send to a worker, along with any static data
type Job = Box<dyn FnOnce() + Send + 'static>;


impl ThreadPool
{
	/// Create with the recommended number of threads, otherwise 1
	pub fn new() -> ThreadPool
	{
		// Get the number of threads or 1
		let count = match std::thread::available_parallelism() {
			Ok(count) => count.get(),
			Err(_) => 1,
		};

		// Create sender and receiver for communication
		let (sender, receiver) = std::sync::mpsc::channel();
		let receiver = Arc::new(Mutex::new(receiver));

		// Create threads
		let mut threads = Vec::with_capacity(count);
		for _i in 0..count {
			threads.push(Thread::new(Arc::clone(&receiver)));
		}

		return ThreadPool {
			threads,
			sender: Some(sender),
		};
	}


	/// Execute the closure with an available thread
	pub fn execute<F>(&self, function: F)
	where
		F: FnOnce() + Send + 'static,
	{
		match self.sender.as_ref() {
			Some(reference) => { let _ = reference.send(Box::new(function)); },
			None => {},
		}
	}
}


impl Drop for ThreadPool
{
	/// Wait for the threads to finish
	fn drop(&mut self)
	{
		drop(self.sender.take());

		for thread in &mut self.threads {
			if let Some(thread) = thread.thread.take() {
				let _ = thread.join();
			}
		}
	}
}


impl Thread
{
	/// Create from a job receiver
	fn new(receiver: Arc<Mutex<Receiver<Job>>>) -> Thread
	{
		let thread = std::thread::spawn(move || loop {
			// Lock the receiver until the first job is taken
			let job = {
				let lock_result = match receiver.lock() {
					Ok(lock_result) => lock_result,
					Err(_) => break,
				};
				match lock_result.recv() {
					Ok(receive_result) => receive_result,
					Err(_) => break,
				}
			};

			// Execute the job
			job();
		});

		return Thread { thread: Some(thread) };
	}
}
