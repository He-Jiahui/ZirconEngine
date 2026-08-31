use std::fmt;

use super::super::TaskPoolBuildError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineTaskGraphOptions {
    worker_threads: usize,
}

impl EngineTaskGraphOptions {
    pub fn with_worker_threads(worker_threads: usize) -> Self {
        Self {
            worker_threads: worker_threads.max(1),
        }
    }

    pub const fn worker_threads(self) -> usize {
        self.worker_threads
    }
}

impl Default for EngineTaskGraphOptions {
    fn default() -> Self {
        Self::with_worker_threads(
            std::thread::available_parallelism().map_or(1, |parallelism| parallelism.get()),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineTaskGraphInitError {
    WorkerSet(TaskPoolBuildError),
}

impl fmt::Display for EngineTaskGraphInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerSet(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for EngineTaskGraphInitError {}

impl From<TaskPoolBuildError> for EngineTaskGraphInitError {
    fn from(error: TaskPoolBuildError) -> Self {
        Self::WorkerSet(error)
    }
}
