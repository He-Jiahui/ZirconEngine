//! Compatibility facade for compute work scheduled on the runtime task pools.

use std::fmt;

use crate::core::tasks::{TaskPool, TaskPoolDescriptor};

#[derive(Clone)]
pub struct JobScheduler {
    pool: TaskPool,
}

impl fmt::Debug for JobScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobScheduler")
            .field("parallelism", &self.parallelism())
            .finish()
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::from_pool(TaskPool::new(TaskPoolDescriptor::compute()))
    }
}

impl JobScheduler {
    pub(crate) fn from_pool(pool: TaskPool) -> Self {
        Self { pool }
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
        self.pool.join(task_a, task_b)
    }

    pub fn parallelism(&self) -> usize {
        self.pool.parallelism()
    }
}
