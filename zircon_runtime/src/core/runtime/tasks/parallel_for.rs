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

impl ParallelSliceExecutor for TaskPool {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        parallel_for_impl(self, items, chunk_size, task);
    }
}

fn parallel_for_impl<T, F>(pool: &TaskPool, items: &mut [T], chunk_size: usize, task: F)
where
    T: Send,
    F: Fn(&mut [T]) + Send + Sync,
{
    pool.install(|| {
        items
            .par_chunks_mut(chunk_size.max(1))
            .for_each(|chunk| task(chunk));
    });
}
