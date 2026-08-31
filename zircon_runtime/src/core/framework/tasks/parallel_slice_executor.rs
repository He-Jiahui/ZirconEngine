/// Neutral blocking slice-parallelism contract for framework algorithms.
///
/// Runtime execution owners implement this contract with their budgeted task
/// pools. Framework code must not create or reach a process-global worker pool.
pub trait ParallelSliceExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync;

    /// Maps a stable range of work indices and preserves source-index order
    /// in the returned output. Executors that only implement mutable slices
    /// retain this bounded serial default; runtime task pools override it.
    fn parallel_map_indices<T, F>(&self, item_count: usize, task: F) -> Vec<T>
    where
        T: Send,
        F: Fn(usize) -> T + Send + Sync,
    {
        (0..item_count).map(task).collect()
    }

    /// Consumes independent work items and preserves their input order in the output.
    ///
    /// Ownership lets execution backends move resource-bearing plans into workers instead of
    /// cloning them merely to cross a neutral framework boundary. The default remains serial for
    /// deterministic tools and test executors.
    fn parallel_map_ordered<T, R, F>(&self, items: Vec<T>, task: F) -> Vec<R>
    where
        T: Send,
        R: Send,
        F: Fn(T) -> R + Send + Sync,
    {
        items.into_iter().map(task).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ParallelSliceExecutor;

    struct SerialExecutor;

    impl ParallelSliceExecutor for SerialExecutor {
        fn parallel_for<T, F>(&self, items: &mut [T], _chunk_size: usize, task: F)
        where
            T: Send,
            F: Fn(&mut [T]) + Send + Sync,
        {
            task(items);
        }
    }

    struct MoveOnly(Box<str>);

    #[test]
    fn ordered_map_default_moves_items_and_preserves_input_order() {
        let values = vec![MoveOnly("first".into()), MoveOnly("second".into())];

        let output = SerialExecutor.parallel_map_ordered(values, |value| value.0);

        assert_eq!(output, vec![Box::<str>::from("first"), Box::from("second")]);
    }
}
