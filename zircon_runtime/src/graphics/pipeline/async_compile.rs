use std::collections::HashSet;
use std::hash::Hash;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::thread::JoinHandle;

#[cfg(test)]
use std::sync::{mpsc::Sender, Arc, Mutex};

type PipelineCompileJob<R> = Box<dyn FnOnce() -> R + Send + 'static>;

struct PipelineCompileRequest<K, R> {
    key: K,
    job: PipelineCompileJob<R>,
}

struct PipelineCompileCompletion<K, R> {
    key: K,
    result: Result<R, PipelineAsyncCompileError>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PipelinePlaceholderPolicy {
    #[default]
    SkipDraw,
    DepthOnly,
}

impl PipelinePlaceholderPolicy {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::SkipDraw => "skip_draw",
            Self::DepthOnly => "depth_only",
        }
    }
}

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

/// Bounded compile worker. The frame path uses `drain_ready`; `finish_pending`
/// is reserved for startup prewarm and explicit synchronization points.
pub(crate) struct PipelineAsyncCompiler<K, R> {
    request_sender: Option<SyncSender<PipelineCompileRequest<K, R>>>,
    completion_receiver: Receiver<PipelineCompileCompletion<K, R>>,
    pending: HashSet<K>,
    max_in_flight: usize,
    worker: Option<JoinHandle<()>>,
    #[cfg(test)]
    target_sync_wait_observer: Option<SyncSender<()>>,
    #[cfg(test)]
    completion_observer: Arc<Mutex<Option<Sender<()>>>>,
}

impl<K, R> PipelineAsyncCompiler<K, R>
where
    K: Clone + Eq + Hash + Send + 'static,
    R: Send + 'static,
{
    pub(crate) fn new(worker_name: &str, max_in_flight: usize) -> std::io::Result<Self> {
        let max_in_flight = max_in_flight.max(1);
        let (request_sender, request_receiver) =
            sync_channel::<PipelineCompileRequest<K, R>>(max_in_flight);
        let (completion_sender, completion_receiver) = channel::<PipelineCompileCompletion<K, R>>();
        #[cfg(test)]
        let completion_observer = Arc::new(Mutex::new(None));
        #[cfg(test)]
        let worker_completion_observer = Arc::clone(&completion_observer);
        let worker = std::thread::Builder::new()
            .name(format!("zircon-{worker_name}"))
            .spawn(move || {
                while let Ok(request) = request_receiver.recv() {
                    let result = catch_unwind(AssertUnwindSafe(request.job))
                        .map_err(|_| PipelineAsyncCompileError::JobPanicked);
                    if completion_sender
                        .send(PipelineCompileCompletion {
                            key: request.key,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                    #[cfg(test)]
                    if let Ok(mut observer) = worker_completion_observer.lock() {
                        if let Some(observer) = observer.take() {
                            let _ = observer.send(());
                        }
                    }
                }
            })?;
        Ok(Self {
            request_sender: Some(request_sender),
            completion_receiver,
            pending: HashSet::with_capacity(max_in_flight),
            max_in_flight,
            worker: Some(worker),
            #[cfg(test)]
            target_sync_wait_observer: None,
            #[cfg(test)]
            completion_observer,
        })
    }

    /// Avoids expensive pipeline source preparation when the bounded worker
    /// cannot accept another background job yet.
    pub(crate) fn has_available_slot(&self) -> bool {
        self.pending.len() < self.max_in_flight
    }

    pub(crate) fn try_queue(
        &mut self,
        key: K,
        job: impl FnOnce() -> R + Send + 'static,
    ) -> PipelineAsyncQueueResult {
        if self.pending.contains(&key) {
            return PipelineAsyncQueueResult::AlreadyPending;
        }
        if self.pending.len() >= self.max_in_flight {
            return PipelineAsyncQueueResult::Full;
        }
        let Some(sender) = &self.request_sender else {
            return PipelineAsyncQueueResult::WorkerUnavailable;
        };
        self.pending.insert(key.clone());
        match sender.try_send(PipelineCompileRequest {
            key: key.clone(),
            job: Box::new(job),
        }) {
            Ok(()) => PipelineAsyncQueueResult::Queued,
            Err(TrySendError::Full(_)) => {
                self.pending.remove(&key);
                PipelineAsyncQueueResult::Full
            }
            Err(TrySendError::Disconnected(_)) => {
                self.pending.remove(&key);
                PipelineAsyncQueueResult::WorkerUnavailable
            }
        }
    }

    pub(crate) fn drain_ready(
        &mut self,
        mut on_ready: impl FnMut(K, Result<R, PipelineAsyncCompileError>),
    ) -> usize {
        let mut drained = 0;
        loop {
            match self.completion_receiver.try_recv() {
                Ok(completion) => {
                    self.pending.remove(&completion.key);
                    on_ready(completion.key, completion.result);
                    drained += 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    for key in self.pending.drain() {
                        on_ready(key, Err(PipelineAsyncCompileError::WorkerUnavailable));
                        drained += 1;
                    }
                    break;
                }
            }
        }
        drained
    }

    pub(crate) fn finish_pending(
        &mut self,
        mut on_ready: impl FnMut(K, Result<R, PipelineAsyncCompileError>),
    ) -> usize {
        let mut completed = self.drain_ready(&mut on_ready);
        while !self.pending.is_empty() {
            match self.completion_receiver.recv() {
                Ok(completion) => {
                    self.pending.remove(&completion.key);
                    on_ready(completion.key, completion.result);
                    completed += 1;
                }
                Err(_) => {
                    for key in self.pending.drain() {
                        on_ready(key, Err(PipelineAsyncCompileError::WorkerUnavailable));
                        completed += 1;
                    }
                }
            }
        }
        completed
    }

    /// Synchronizes only through `target` in the worker's FIFO stream.
    ///
    /// This is for startup prewarm of one required pipeline. Later queued work
    /// remains pending so a startup synchronization point does not drain the
    /// full async compilation budget.
    pub(crate) fn finish_pending_through(
        &mut self,
        target: &K,
        mut on_ready: impl FnMut(K, Result<R, PipelineAsyncCompileError>),
    ) -> usize {
        let mut completed = self.drain_ready(&mut on_ready);
        #[cfg(test)]
        if self.pending.contains(target) {
            if let Some(observer) = self.target_sync_wait_observer.take() {
                let _ = observer.send(());
            }
        }
        while self.pending.contains(target) {
            match self.completion_receiver.recv() {
                Ok(completion) => {
                    self.pending.remove(&completion.key);
                    on_ready(completion.key, completion.result);
                    completed += 1;
                }
                Err(_) => {
                    for key in self.pending.drain() {
                        on_ready(key, Err(PipelineAsyncCompileError::WorkerUnavailable));
                        completed += 1;
                    }
                }
            }
        }
        completed
    }

    #[cfg(test)]
    pub(crate) fn set_target_sync_wait_observer(&mut self, observer: SyncSender<()>) {
        self.target_sync_wait_observer = Some(observer);
    }

    #[cfg(test)]
    pub(crate) fn set_completion_observer(&mut self, observer: Sender<()>) {
        if let Ok(mut completion_observer) = self.completion_observer.lock() {
            *completion_observer = Some(observer);
        }
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn is_pending(&self, key: &K) -> bool {
        self.pending.contains(key)
    }
}

impl<K, R> Drop for PipelineAsyncCompiler<K, R> {
    fn drop(&mut self) {
        self.request_sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PipelineAsyncCompileError, PipelineAsyncCompiler, PipelineAsyncQueueResult,
        PipelinePlaceholderPolicy,
    };

    #[test]
    fn render_perf_async_pipeline_queue_deduplicates_and_recovers_completion() {
        let mut compiler = PipelineAsyncCompiler::new("pipeline-test", 2).unwrap();
        assert_eq!(
            compiler.try_queue(7_u32, || 41_u32),
            PipelineAsyncQueueResult::Queued
        );
        assert_eq!(
            compiler.try_queue(7_u32, || 99_u32),
            PipelineAsyncQueueResult::AlreadyPending
        );

        let mut completed = Vec::new();
        compiler.finish_pending(|key, result| completed.push((key, result)));

        assert_eq!(completed, vec![(7, Ok(41))]);
        assert_eq!(compiler.pending_count(), 0);
    }

    #[test]
    fn render_perf_async_pipeline_queue_has_a_hard_in_flight_budget() {
        let mut compiler = PipelineAsyncCompiler::new("bounded-pipeline-test", 1).unwrap();
        assert!(compiler.has_available_slot());
        assert_eq!(
            compiler.try_queue(1_u32, || 1_u32),
            PipelineAsyncQueueResult::Queued
        );
        assert!(!compiler.has_available_slot());
        assert_eq!(
            compiler.try_queue(2_u32, || 2_u32),
            PipelineAsyncQueueResult::Full
        );
        compiler.finish_pending(|_, _| {});
        assert!(compiler.has_available_slot());
    }

    #[test]
    fn render_perf_async_pipeline_target_sync_leaves_later_work_pending() {
        let mut compiler = PipelineAsyncCompiler::new("target-sync-test", 3).unwrap();
        let (release_first, wait_first) = std::sync::mpsc::sync_channel(0);
        let (release_later, wait_later) = std::sync::mpsc::sync_channel(0);
        assert_eq!(
            compiler.try_queue(1_u32, move || {
                wait_first.recv().expect("test releases the first job");
                1_u32
            }),
            PipelineAsyncQueueResult::Queued
        );
        assert_eq!(
            compiler.try_queue(2_u32, || 2_u32),
            PipelineAsyncQueueResult::Queued
        );
        assert_eq!(
            compiler.try_queue(3_u32, move || {
                wait_later.recv().expect("test releases the later job");
                3_u32
            }),
            PipelineAsyncQueueResult::Queued
        );

        release_first
            .send(())
            .expect("first worker job should still be waiting");
        let mut completed = Vec::new();
        assert_eq!(
            compiler.finish_pending_through(&2, |key, result| completed.push((key, result))),
            2
        );

        assert_eq!(completed, vec![(1, Ok(1)), (2, Ok(2))]);
        assert!(compiler.is_pending(&3));
        release_later
            .send(())
            .expect("later worker job should still be waiting");
        compiler.finish_pending(|_, _| {});
    }

    #[test]
    fn render_perf_async_pipeline_worker_contains_job_panics() {
        let mut compiler =
            PipelineAsyncCompiler::<u32, u32>::new("panic-pipeline-test", 1).unwrap();
        compiler.try_queue(3, || panic!("synthetic compile panic"));

        let mut completed = Vec::new();
        compiler.finish_pending(|key, result| completed.push((key, result)));

        assert_eq!(
            completed,
            vec![(3, Err(PipelineAsyncCompileError::JobPanicked))]
        );
    }

    #[test]
    fn render_perf_async_pipeline_placeholder_no_error_material() {
        assert_eq!(
            PipelinePlaceholderPolicy::default(),
            PipelinePlaceholderPolicy::SkipDraw
        );
        assert_eq!(PipelinePlaceholderPolicy::DepthOnly.label(), "depth_only");
    }
}
