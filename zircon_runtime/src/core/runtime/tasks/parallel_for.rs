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
    match item_count {
        0 => Vec::new(),
        1 => vec![task(0)],
        _ => pool.install(|| (0..item_count).into_par_iter().map(task).collect()),
    }
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

#[cfg(test)]
#[path = "parallel_for/tests/optimization_batch_jt_runtime659_tests.rs"]
mod optimization_batch_jt_runtime659_tests;

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

// 空输入直接返回；单块只在指定池内调用一次回调，多块才使用并行分块迭代，避免额外拆分调度。
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
#[path = "tests/parallel_for.rs"]
mod tests;
