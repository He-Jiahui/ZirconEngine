use std::fmt;
use std::sync::Arc;

use rayon::ThreadPool;

use super::{TaskPoolDescriptor, TaskPoolKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskPoolYield {
    Executed,
    Idle,
}

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

    pub fn shares_execution_owner_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.pool, &other.pool)
    }

    pub(crate) fn is_current_worker(&self) -> bool {
        self.pool.current_thread_index().is_some()
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.pool.spawn(task);
    }

    pub fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        self.pool.install(task)
    }

    pub(crate) fn in_place_scope<'scope, OP, R>(&self, task: OP) -> R
    where
        OP: FnOnce(&rayon::Scope<'scope>) -> R,
    {
        self.pool.in_place_scope(task)
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

pub(super) fn assist_current_thread_once() -> Option<TaskPoolYield> {
    rayon::yield_now().map(|result| match result {
        rayon::Yield::Executed => TaskPoolYield::Executed,
        rayon::Yield::Idle => TaskPoolYield::Idle,
    })
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
