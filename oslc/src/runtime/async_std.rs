use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

pub struct Task {
    id: usize,
    func: Box<dyn FnOnce() + Send + 'static>,
}

pub struct Runtime {
    tasks: Arc<Mutex<VecDeque<Task>>>,
    workers: usize,
}

impl Runtime {
    pub fn new(workers: usize) -> Self {
        Runtime {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            workers,
        }
    }
    
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task {
            id: 0,
            func: Box::new(f),
        };
        self.tasks.lock().unwrap().push_back(task);
    }
    
    pub fn run(&self) {
        let mut handles = Vec::new();
        for _ in 0..self.workers {
            let tasks = Arc::clone(&self.tasks);
            let handle = thread::spawn(move || {
                loop {
                    let task = tasks.lock().unwrap().pop_front();
                    if let Some(t) = task {
                        (t.func)();
                    } else {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            });
            handles.push(handle);
        }
        for h in handles {
            h.join().ok();
        }
    }
}

pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let _ = f;
    println!("Task spawned");
}

pub struct LazyRuntime(std::sync::OnceLock<Runtime>);

impl LazyRuntime {
    const fn new() -> Self {
        LazyRuntime(std::sync::OnceLock::new())
    }
    
    fn get(&self) -> &Runtime {
        self.0.get_or_init(|| Runtime::new(4))
    }
}

static RUNTIME: LazyRuntime = LazyRuntime::new();

pub struct Channel<T> {
    sender: Arc<Mutex<VecDeque<T>>>,
    receiver: Arc<Mutex<VecDeque<T>>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Channel {
            sender: Arc::new(Mutex::new(VecDeque::new())),
            receiver: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    pub fn send(&self, value: T) {
        self.sender.lock().unwrap().push_back(value);
    }
    
    pub fn recv(&self) -> Option<T> {
        self.sender.lock().unwrap().pop_front()
    }
}

pub fn sleep(duration: Duration) {
    thread::sleep(duration);
}
