use std::fmt;
use std::sync::Arc;

use rayon::ThreadPool;

use super::{TaskPoolDescriptor, TaskPoolKind};

#[derive(Clone)]
pub struct TaskPool {
    descriptor: TaskPoolDescriptor,
    parallelism: usize,
    pool: Arc<ThreadPool>,
}

impl TaskPool {
    pub fn new(descriptor: TaskPoolDescriptor) -> Self {
        let parallelism = descriptor
            .worker_threads
            .unwrap_or_else(default_parallelism)
            .max(1);
        let thread_name = descriptor.thread_name.clone();
        let pool = rayon::ThreadPoolBuilder::new()
            .thread_name(move |index| format!("{thread_name}-{index}"))
            .num_threads(parallelism)
            .build()
            .expect("zircon task pool");
        Self {
            descriptor,
            parallelism,
            pool: Arc::new(pool),
        }
    }

    pub fn kind(&self) -> TaskPoolKind {
        self.descriptor.kind
    }

    pub fn descriptor(&self) -> &TaskPoolDescriptor {
        &self.descriptor
    }

    pub fn parallelism(&self) -> usize {
        self.parallelism
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.pool.spawn(task);
    }

    pub fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        self.pool.install(task)
    }

    pub fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.pool.install(|| rayon::join(task_a, task_b))
    }
}

impl fmt::Debug for TaskPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskPool")
            .field("kind", &self.descriptor.kind)
            .field("thread_name", &self.descriptor.thread_name)
            .field("parallelism", &self.parallelism)
            .finish()
    }
}

fn default_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, |value| value.get())
}
