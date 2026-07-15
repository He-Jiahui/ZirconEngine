/// Neutral blocking slice-parallelism contract for framework algorithms.
///
/// Runtime execution owners implement this contract with their budgeted task
/// pools. Framework code must not create or reach a process-global worker pool.
pub trait ParallelSliceExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync;
}
