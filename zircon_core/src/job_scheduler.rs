//! Rayon-backed background job pool.

use std::fmt;
use std::sync::Arc;

use rayon::ThreadPool;

#[derive(Clone)]
pub struct JobScheduler {
    parallelism: usize,
    pool: Arc<ThreadPool>,
}

impl fmt::Debug for JobScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobScheduler")
            .field("parallelism", &self.parallelism)
            .finish()
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism().map_or(1, |value| value.get());
        let pool = rayon::ThreadPoolBuilder::new()
            .thread_name(|index| format!("zircon-job-{index}"))
            .num_threads(parallelism.max(1))
            .build()
            .expect("job scheduler thread pool");
        Self {
            parallelism,
            pool: Arc::new(pool),
        }
    }
}

impl JobScheduler {
    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.pool.spawn(task);
    }

    pub fn parallelism(&self) -> usize {
        self.parallelism
    }
}
