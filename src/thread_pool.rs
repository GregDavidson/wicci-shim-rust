// from Rust Book Final Project

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

enum Message {
  NewJob(Job),
  Terminate,
}

pub struct ThreadPool {
  num_workers: usize,
  max_workers: usize,
  wait_milliseconds: Duration, // time to wait for a worker to become available when we have max_workers
  sender: mpsc::Sender<Message>,
}

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>) {
    (*self)()
  }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  pub fn new(max_workers: usize, wait_milliseconds: Duration) -> ThreadPool {
    assert!(max_workers > 0);

    let (sender, receiver) = mpsc::sync_channel(0); // rendezvous channel
    let receiver = Arc::new(Mutex::new(receiver));
    
    ThreadPool { num_workers: 0, max_workers, wait_milliseconds, sender }
  }

  // reasons why an execute might fail
  enum ExecuteFail {
    WeAreBusy,                   // pool is at max and all workers busy
    Disconnected                 // wait, how could this happen, to us??
  };
  
  pub fn execute<F>(&self, f: F) -> Result( (), ExecuteFail )
    where  F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);
    
    loop {
      match self.sender.try_send(Message::NewJob(job)) {
        Ok(_) => { // rendezvous successful, package delivered
          if self.sender.try_send(Message::NewJob(job)) {
            return Ok( () )
          },
          Err(e) => match e {
            Full => { // no worker available
              if self.num_workers < self.max_workers { // spawn a new worker
                let _ = Worker::new(id, job, Arc::clone(&receiver));
                ++self.num_workers;
                return ();
              } else {
                // sleep and try again
                thread::sleep(Duration::from_millis(wait_milliseconds));
              }
            },
            Disconnected => {
              return Disconnected;
            }
          }
        }
      }
    }
  }

impl Drop for ThreadPool {
  fn drop(&mut self) {
    eprintln!("Sending terminate message to all workers.");
    
    eprintln!("Shutting down all workers.");
    for _ in self.num_workers {
      self.sender.send(Message::Terminate).unwrap();
    }
    eprintln!("All workers shut down.");
  }
}

struct Worker {
  id: usize,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, job: Job, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    
    let thread = thread::spawn(move ||{
      job.call_box();           // run our first job
      loop {
        let message = receiver.lock().unwrap().recv().unwrap();
        
        match message {
          Message::NewJob(job) => {
            eprintln!("Worker {} got a job; executing.", id);
            job.call_box();
          },
          Message::Terminate => {
            eprintln!("Worker {} was told to terminate.", id);
            break;
          },
        }
      }
    });
    
    Worker { id, thread: Some(thread) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn do_something() {
    // do something!
  }
}
