use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub payload: String,
    pub status: JobStatus,
    pub retry: u32,
    pub max_retries: u32,
    pub scheduled_at: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

pub struct JobQueue {
    queues: Arc<Mutex<HashMap<String, VecDeque<Job>>>>,
    results: Arc<Mutex<HashMap<String, Job>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        JobQueue {
            queues: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn enqueue(&self, queue: &str, payload: &str, delay: u64) -> String {
        let id = format!("job_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        let job = Job {
            id: id.clone(),
            payload: payload.to_string(),
            status: JobStatus::Pending,
            retry: 0,
            max_retries: 3,
            scheduled_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + delay,
        };
        self.queues.lock().unwrap()
            .entry(queue.to_string())
            .or_insert_with(VecDeque::new)
            .push_back(job);
        id
    }
    
    pub fn dequeue(&self, queue: &str) -> Option<Job> {
        let mut queues = self.queues.lock().unwrap();
        if let Some(q) = queues.get_mut(queue) {
            q.pop_front()
        } else {
            None
        }
    }
    
    pub fn complete(&self, job_id: &str) {
        let mut results = self.results.lock().unwrap();
        if let Some(job) = results.get_mut(job_id) {
            job.status = JobStatus::Completed;
        }
    }
    
    pub fn fail(&self, job_id: &str) {
        let mut results = self.results.lock().unwrap();
        if let Some(job) = results.get_mut(job_id) {
            if job.retry < job.max_retries {
                job.retry += 1;
                job.status = JobStatus::Pending;
            } else {
                job.status = JobStatus::Failed;
            }
        }
    }
    
    pub fn status(&self, job_id: &str) -> Option<JobStatus> {
        self.results.lock().unwrap().get(job_id).map(|j| j.status.clone())
    }
}

pub struct Worker {
    queue: JobQueue,
    name: String,
}

impl Worker {
    pub fn new(name: &str) -> Self {
        Worker {
            queue: JobQueue::new(),
            name: name.to_string(),
        }
    }
    
    pub fn process<F>(&self, queue: &str, handler: F) where F: Fn(String) {
        while let Some(job) = self.queue.dequeue(queue) {
            handler(job.payload);
            self.queue.complete(&job.id);
        }
    }
}
