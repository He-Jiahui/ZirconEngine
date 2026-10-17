//! Single-thread PSO work is bounded and advances through the existing frame polling API.
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
    panic::{catch_unwind, AssertUnwindSafe},
};
type Job<R> = Box<dyn FnOnce() -> R + Send + 'static>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineAsyncQueueResult {
    Queued,
    AlreadyPending,
    Full,
    WorkerUnavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineAsyncCompileError {
    JobPanicked,
    WorkerUnavailable,
}
pub(crate) struct PipelineAsyncCompiler<K, R> {
    jobs: VecDeque<(K, Job<R>)>,
    pending: HashSet<K>,
    capacity: usize,
}
impl<K: Clone + Eq + Hash + Send + 'static, R: Send + 'static> PipelineAsyncCompiler<K, R> {
    pub(crate) fn new(_name: &str, capacity: usize) -> std::io::Result<Self> {
        if capacity > 64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "browser pipeline queue limit is 64",
            ));
        }
        let capacity = capacity.max(1);
        Ok(Self {
            jobs: VecDeque::with_capacity(capacity),
            pending: HashSet::with_capacity(capacity),
            capacity,
        })
    }
    pub(crate) fn has_available_slot(&self) -> bool {
        self.pending.len() < self.capacity
    }
    pub(crate) fn is_pending(&self, key: &K) -> bool {
        self.pending.contains(key)
    }
    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub(crate) fn try_queue(
        &mut self,
        key: K,
        job: impl FnOnce() -> R + Send + 'static,
    ) -> PipelineAsyncQueueResult {
        if self.pending.contains(&key) {
            return PipelineAsyncQueueResult::AlreadyPending;
        }
        if !self.has_available_slot() {
            return PipelineAsyncQueueResult::Full;
        }
        self.pending.insert(key.clone());
        self.jobs.push_back((key, Box::new(job)));
        PipelineAsyncQueueResult::Queued
    }
    pub(crate) fn drain_ready(
        &mut self,
        mut ready: impl FnMut(K, Result<R, PipelineAsyncCompileError>),
    ) -> usize {
        let Some((key, job)) = self.jobs.pop_front() else {
            return 0;
        };
        self.pending.remove(&key);
        ready(
            key,
            catch_unwind(AssertUnwindSafe(job)).map_err(|_| PipelineAsyncCompileError::JobPanicked),
        );
        1
    }
    pub(crate) fn finish_pending(
        &mut self,
        mut ready: impl FnMut(K, Result<R, PipelineAsyncCompileError>),
    ) -> usize {
        let mut completed = 0;
        while !self.jobs.is_empty() {
            completed += self.drain_ready(&mut ready);
        }
        completed
    }
    pub(crate) fn finish_pending_through(
        &mut self,
        target: &K,
        mut ready: impl FnMut(K, Result<R, PipelineAsyncCompileError>),
    ) -> usize {
        let mut completed = 0;
        while self.pending.contains(target) {
            completed += self.drain_ready(&mut ready);
        }
        completed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn web_compile_queue_is_bounded_deduplicated_and_preserves_fifo() {
        let mut compiler = PipelineAsyncCompiler::new("test", 2).unwrap();
        assert_eq!(
            compiler.try_queue(1, || 10),
            PipelineAsyncQueueResult::Queued
        );
        assert_eq!(
            compiler.try_queue(1, || 999),
            PipelineAsyncQueueResult::AlreadyPending
        );
        assert_eq!(
            compiler.try_queue(2, || 20),
            PipelineAsyncQueueResult::Queued
        );
        assert_eq!(compiler.try_queue(3, || 30), PipelineAsyncQueueResult::Full);
        let mut out = Vec::new();
        assert_eq!(
            compiler.drain_ready(|key, value| out.push((key, value.unwrap()))),
            1
        );
        assert_eq!(out, [(1, 10)]);
        assert_eq!(
            compiler.try_queue(3, || 30),
            PipelineAsyncQueueResult::Queued
        );
        assert_eq!(
            compiler.finish_pending_through(&2, |key, value| out.push((key, value.unwrap()))),
            1
        );
        assert_eq!(out, [(1, 10), (2, 20)]);
        assert_eq!(compiler.pending_count(), 1);
        compiler.finish_pending(|key, value| out.push((key, value.unwrap())));
        assert_eq!(out, [(1, 10), (2, 20), (3, 30)]);
        assert!(!compiler.is_pending(&3));
    }
    #[test]
    fn web_compile_drop_cancels_queued_work_and_releases_its_inputs() {
        let input = std::sync::Arc::new(());
        let mut compiler = PipelineAsyncCompiler::new("test", 1).unwrap();
        let owned = input.clone();
        compiler.try_queue(1, move || {
            drop(owned);
            0
        });
        assert_eq!(std::sync::Arc::strong_count(&input), 2);
        drop(compiler);
        assert_eq!(std::sync::Arc::strong_count(&input), 1);
    }
}
