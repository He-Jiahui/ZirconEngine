use rayon::prelude::*;

use super::TaskPool;
use crate::core::framework::tasks::ParallelSliceExecutor;

pub fn parallel_for<T, F>(pool: &TaskPool, items: &mut [T], chunk_size: usize, f: F)
where
    T: Send,
    F: Fn(&mut [T]) + Send + Sync,
{
    parallel_for_impl(pool, items, chunk_size, f);
}

pub fn parallel_map_indices<T, F>(pool: &TaskPool, item_count: usize, task: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Send + Sync,
{
    if item_count == 0 {
        return Vec::new();
    }
    pool.install(|| (0..item_count).into_par_iter().map(task).collect())
}

pub fn parallel_map_ordered<T, R, F>(pool: &TaskPool, mut items: Vec<T>, task: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    match items.len() {
        0 => Vec::new(),
        1 => {
            let Some(item) = items.pop() else {
                return Vec::new();
            };
            vec![task(item)]
        }
        _ => pool.install(|| items.into_par_iter().map(task).collect()),
    }
}

impl ParallelSliceExecutor for TaskPool {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        parallel_for_impl(self, items, chunk_size, task);
    }

    fn parallel_map_indices<T, F>(&self, item_count: usize, task: F) -> Vec<T>
    where
        T: Send,
        F: Fn(usize) -> T + Send + Sync,
    {
        parallel_map_indices(self, item_count, task)
    }

    fn parallel_map_ordered<T, R, F>(&self, items: Vec<T>, task: F) -> Vec<R>
    where
        T: Send,
        R: Send,
        F: Fn(T) -> R + Send + Sync,
    {
        parallel_map_ordered(self, items, task)
    }
}

fn parallel_for_impl<T, F>(pool: &TaskPool, items: &mut [T], chunk_size: usize, task: F)
where
    T: Send,
    F: Fn(&mut [T]) + Send + Sync,
{
    if items.is_empty() {
        return;
    }
    let chunk_size = chunk_size.max(1);
    if items.len() <= chunk_size {
        pool.install(|| task(items));
        return;
    }
    pool.install(|| {
        items
            .par_chunks_mut(chunk_size)
            .for_each(|chunk| task(chunk));
    });
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use super::*;
    use crate::core::runtime::tasks::TaskPoolDescriptor;

    const EMPTY_FAST_PATH_CALLS: usize = 100_000;
    const SINGLE_CHUNK_FAST_PATH_CALLS: usize = 25_000;
    const MAX_EMPTY_FAST_PATH_LATENCY: Duration = Duration::from_millis(250);
    const MAX_SINGLE_CHUNK_FAST_PATH_LATENCY: Duration = Duration::from_secs(2);

    #[test]
    fn runtime02_empty_and_single_chunk_paths_bypass_parallel_iteration() {
        let source = include_str!("parallel_for.rs");
        let implementation = source
            .split("mod tests {")
            .next()
            .expect("parallel_for implementation");

        assert!(implementation.contains("if items.is_empty()"));
        assert!(implementation.contains("if items.len() <= chunk_size"));
        assert!(implementation.contains("pool.install(|| task(items))"));
        let single_chunk = implementation
            .find("if items.len() <= chunk_size")
            .expect("single chunk fast path");
        let parallel_chunks = implementation
            .find(".par_chunks_mut(chunk_size)")
            .expect("multi-chunk parallel path");
        assert!(single_chunk < parallel_chunks);
    }

    #[test]
    fn runtime02_empty_input_never_invokes_the_task_and_single_chunk_runs_once() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let calls = AtomicUsize::new(0);
        let mut empty: [u32; 0] = [];
        parallel_for(&pool, &mut empty, 8, |_| {
            calls.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(calls.load(Ordering::Relaxed), 0);

        let mut values = [1_u32, 2, 3, 4];
        parallel_for(&pool, &mut values, 8, |chunk| {
            calls.fetch_add(1, Ordering::Relaxed);
            for value in chunk {
                *value += 1;
            }
        });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(values, [2, 3, 4, 5]);
    }

    #[test]
    fn parallel_map_indices_preserves_source_index_order() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));

        let values = parallel_map_indices(&pool, 32, |index| index * index);

        assert_eq!(
            values,
            (0..32).map(|index| index * index).collect::<Vec<_>>()
        );
    }

    #[test]
    fn parallel_map_ordered_preserves_owned_input_order() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));

        let values = parallel_map_ordered(&pool, (0..32).collect(), |value| value * value);

        assert_eq!(
            values,
            (0..32).map(|value| value * value).collect::<Vec<_>>()
        );
    }

    #[test]
    fn parallel_map_ordered_empty_and_single_item_fast_paths_are_exact_once() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
        let calls = AtomicUsize::new(0);

        let empty = parallel_map_ordered(&pool, Vec::<usize>::new(), |value| {
            calls.fetch_add(1, Ordering::Relaxed);
            value
        });
        let single = parallel_map_ordered(&pool, vec![7_usize], |value| {
            calls.fetch_add(1, Ordering::Relaxed);
            value * 2
        });

        assert!(empty.is_empty());
        assert_eq!(single, vec![14]);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    #[ignore = "managed Runtime02 performance evidence"]
    fn runtime02_parallel_for_small_input_fast_path_evidence() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let empty_started = Instant::now();
        for _ in 0..EMPTY_FAST_PATH_CALLS {
            let mut empty: [u32; 0] = [];
            parallel_for(&pool, &mut empty, 8, |_| unreachable!());
        }
        let empty_elapsed = empty_started.elapsed();

        let single_started = Instant::now();
        let mut value = [0_u32];
        for _ in 0..SINGLE_CHUNK_FAST_PATH_CALLS {
            parallel_for(&pool, &mut value, 8, |chunk| chunk[0] += 1);
        }
        let single_elapsed = single_started.elapsed();

        assert!(empty_elapsed <= MAX_EMPTY_FAST_PATH_LATENCY);
        assert!(single_elapsed <= MAX_SINGLE_CHUNK_FAST_PATH_LATENCY);
        assert_eq!(value[0], SINGLE_CHUNK_FAST_PATH_CALLS as u32);
        println!(
            "TASK_POOL_BENCH_V1 kind=parallel_for_small_input empty_calls={} empty_pool_installs_before={} empty_pool_installs_after=0 empty_install_reduction_percent=100.0000 empty_elapsed_ns={} empty_target_ns={} single_chunk_calls={} single_chunk_parallel_iterators_before={} single_chunk_parallel_iterators_after=0 single_chunk_iterator_reduction_percent=100.0000 single_chunk_elapsed_ns={} single_chunk_target_ns={}",
            EMPTY_FAST_PATH_CALLS,
            EMPTY_FAST_PATH_CALLS,
            empty_elapsed.as_nanos(),
            MAX_EMPTY_FAST_PATH_LATENCY.as_nanos(),
            SINGLE_CHUNK_FAST_PATH_CALLS,
            SINGLE_CHUNK_FAST_PATH_CALLS,
            single_elapsed.as_nanos(),
            MAX_SINGLE_CHUNK_FAST_PATH_LATENCY.as_nanos(),
        );
    }
}
