use rayon::prelude::*;

use super::TaskPool;

pub fn parallel_for<T, F>(pool: &TaskPool, items: &mut [T], chunk_size: usize, f: F)
where
    T: Send,
    F: Fn(&mut [T]) + Send + Sync,
{
    let chunk_size = chunk_size.max(1);
    pool.install(|| {
        items.par_chunks_mut(chunk_size).for_each(|chunk| f(chunk));
    });
}
