use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::{error, info};
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

pub struct ThreadPooling {
    runtime: Arc<Runtime>,
    thread_count: usize,
}

impl ThreadPooling {
    pub fn new(thread_count: usize) -> Self {
        // Create a multi-threaded runtime with the specified number of threads
        let runtime = Builder::new_multi_thread()
            .worker_threads(thread_count)
            .enable_all()
            .build()
            .expect("Failed to create thread pool runtime");
        
        info!("Thread pool created with {} threads", thread_count);
        
        ThreadPooling {
            runtime: Arc::new(runtime),
            thread_count,
        }
    }
    
    pub fn run<F>(&self, task: F, delay_ms: u64) -> JoinHandle<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let runtime = self.runtime.clone();
        
        // Spawn the task with the specified delay
        runtime.spawn(async move {
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            
            task();
        })
    }
    
    pub fn run_scheduled<F>(&self, task: F, delay_ms: u64, interval_ms: u64) -> JoinHandle<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let runtime = self.runtime.clone();
        let task = Arc::new(task);
        
        // Spawn a scheduled task that runs at the specified interval
        runtime.spawn(async move {
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
            
            loop {
                interval.tick().await;
                
                let task_clone = task.clone();
                task_clone();
                
                // Check if we should stop
                if crate::is_shutting_down() {
                    break;
                }
            }
        })
    }
    
    pub fn get_thread_count(&self) -> usize {
        self.thread_count
    }
    
    pub fn get_runtime(&self) -> Arc<Runtime> {
        self.runtime.clone()
    }
}