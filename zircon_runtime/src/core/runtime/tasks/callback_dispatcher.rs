//! Bounded asynchronous delivery for task completion and lifecycle callbacks.

use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{TaskPool, TaskPools};

pub(super) type TaskCallback = Box<dyn FnOnce() + Send + 'static>;

const MAX_CALLBACKS_PER_RUN: usize = 64;
const MAX_CALLBACKS_PER_ENVELOPE: usize = 8;
const MAX_CONCURRENT_DISPATCH_RUNNERS: usize = 2;

static PROCESS_CALLBACK_DISPATCHER: OnceLock<TaskCallbackDispatcher> = OnceLock::new();

#[derive(Clone)]
pub(super) struct TaskCallbackDispatcher {
    inner: Arc<TaskCallbackDispatcherInner>,
}

struct TaskCallbackDispatcherInner {
    pool: Option<TaskPool>,
    max_active_runners: usize,
    state: Mutex<TaskCallbackDispatcherState>,
    #[cfg(test)]
    metrics: TaskCallbackDispatcherTestMetrics,
}

struct TaskCallbackDispatcherState {
    envelopes: VecDeque<CallbackEnvelope>,
    active_runners: usize,
    inline_draining: bool,
}

struct CallbackEnvelope {
    callbacks: VecDeque<TaskCallback>,
    completion: Option<TaskCallback>,
}

#[cfg(test)]
struct TaskCallbackDispatcherTestMetrics {
    delivery_runs: AtomicUsize,
    delivered_callbacks: AtomicUsize,
    max_callbacks_per_run: AtomicUsize,
    pending_callbacks: AtomicUsize,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TaskCallbackDispatcherMetricsSnapshot {
    delivery_runs: usize,
    delivered_callbacks: usize,
    max_callbacks_per_run: usize,
    pending_callbacks: usize,
}

impl TaskCallbackDispatcher {
    pub(super) fn process_default() -> Self {
        PROCESS_CALLBACK_DISPATCHER
            .get_or_init(|| Self::new(TaskPools::process_default().async_compute().clone()))
            .clone()
    }

    pub(super) fn new(pool: TaskPool) -> Self {
        Self {
            inner: Arc::new(TaskCallbackDispatcherInner {
                max_active_runners: pool.parallelism().min(MAX_CONCURRENT_DISPATCH_RUNNERS),
                pool: Some(pool),
                state: Mutex::new(TaskCallbackDispatcherState {
                    envelopes: VecDeque::new(),
                    active_runners: 0,
                    inline_draining: false,
                }),
                #[cfg(test)]
                metrics: TaskCallbackDispatcherTestMetrics {
                    delivery_runs: AtomicUsize::new(0),
                    delivered_callbacks: AtomicUsize::new(0),
                    max_callbacks_per_run: AtomicUsize::new(0),
                    pending_callbacks: AtomicUsize::new(0),
                },
            }),
        }
    }

    pub(super) fn inline() -> Self {
        Self {
            inner: Arc::new(TaskCallbackDispatcherInner {
                pool: None,
                max_active_runners: 1,
                state: Mutex::new(TaskCallbackDispatcherState {
                    envelopes: VecDeque::new(),
                    active_runners: 0,
                    inline_draining: false,
                }),
                #[cfg(test)]
                metrics: TaskCallbackDispatcherTestMetrics {
                    delivery_runs: AtomicUsize::new(0),
                    delivered_callbacks: AtomicUsize::new(0),
                    max_callbacks_per_run: AtomicUsize::new(0),
                    pending_callbacks: AtomicUsize::new(0),
                },
            }),
        }
    }

    pub(super) fn dispatch(&self, callbacks: Vec<TaskCallback>, completion: Option<TaskCallback>) {
        if callbacks.is_empty() && completion.is_none() {
            return;
        }
        self.record_callbacks_enqueued(callbacks.len() + usize::from(completion.is_some()));
        if self.inner.pool.is_none() {
            self.lock_state().envelopes.push_back(CallbackEnvelope {
                callbacks: callbacks.into(),
                completion,
            });
            self.drain_inline_without_executor();
            return;
        }

        let runners_to_schedule = {
            let mut state = self.lock_state();
            state.envelopes.push_back(CallbackEnvelope {
                callbacks: callbacks.into(),
                completion,
            });
            self.reserve_runners(&mut state)
        };

        for _ in 0..runners_to_schedule {
            self.schedule_runner();
        }
    }

    pub(super) fn dispatch_one(&self, callback: TaskCallback) {
        self.dispatch(Vec::new(), Some(callback));
    }

    fn schedule_runner(&self) {
        let Some(submission) = self
            .inner
            .pool
            .as_ref()
            .and_then(TaskPool::try_acquire_continuation)
        else {
            // A terminal observer may be registered after the owning worker
            // set has joined. The callback is already admitted by JobHandle,
            // so deliver it synchronously instead of escaping to another owner.
            self.drain_inline_without_executor();
            return;
        };
        let dispatcher = self.clone();
        submission.spawn(move || dispatcher.run());
    }

    fn drain_inline_without_executor(&self) {
        {
            let mut state = self.lock_state();
            if state.inline_draining {
                return;
            }
            state.inline_draining = true;
        }

        loop {
            let callbacks_run = self.run_budget();
            self.record_run(callbacks_run);
            let mut state = self.lock_state();
            if state.envelopes.is_empty() {
                state.active_runners = 0;
                state.inline_draining = false;
                return;
            }
        }
    }

    fn run(&self) {
        let callbacks_run = self.run_budget();
        self.record_run(callbacks_run);
        self.finish_run();
    }

    fn run_budget(&self) -> usize {
        let mut callbacks_run = 0;
        while callbacks_run < MAX_CALLBACKS_PER_RUN {
            let Some(mut envelope) = self.pop_envelope() else {
                return callbacks_run;
            };

            let available = (MAX_CALLBACKS_PER_RUN - callbacks_run).min(MAX_CALLBACKS_PER_ENVELOPE);
            let callbacks_delivered = envelope.run(available);
            self.record_callbacks_completed(callbacks_delivered);
            callbacks_run += callbacks_delivered;
            if !envelope.is_complete() {
                // Yield a large fan-out before consuming another envelope's turn.
                self.push_envelope(envelope);
            }
        }
        callbacks_run
    }

    fn pop_envelope(&self) -> Option<CallbackEnvelope> {
        self.lock_state().envelopes.pop_front()
    }

    fn push_envelope(&self, envelope: CallbackEnvelope) {
        self.lock_state().envelopes.push_back(envelope);
    }

    fn finish_run(&self) {
        let runners_to_schedule = {
            let mut state = self.lock_state();
            state.active_runners = state
                .active_runners
                .checked_sub(1)
                .expect("every dispatcher runner must hold a reservation");
            self.reserve_runners(&mut state)
        };
        for _ in 0..runners_to_schedule {
            self.schedule_runner();
        }
    }

    fn reserve_runners(&self, state: &mut TaskCallbackDispatcherState) -> usize {
        if state.inline_draining {
            return 0;
        }
        // Reserve before spawning so producers cannot exceed this dispatcher's pool budget.
        let capacity = self
            .inner
            .max_active_runners
            .saturating_sub(state.active_runners);
        let runners = capacity.min(state.envelopes.len());
        state.active_runners += runners;
        runners
    }

    fn lock_state(&self) -> MutexGuard<'_, TaskCallbackDispatcherState> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    fn record_run(&self, callbacks_run: usize) {
        if callbacks_run == 0 {
            return;
        }
        self.inner
            .metrics
            .delivery_runs
            .fetch_add(1, Ordering::Relaxed);
        self.inner
            .metrics
            .delivered_callbacks
            .fetch_add(callbacks_run, Ordering::Relaxed);
        self.inner
            .metrics
            .max_callbacks_per_run
            .fetch_max(callbacks_run, Ordering::Relaxed);
    }

    #[cfg(not(test))]
    fn record_run(&self, _callbacks_run: usize) {}

    #[cfg(test)]
    fn record_callbacks_enqueued(&self, count: usize) {
        self.inner
            .metrics
            .pending_callbacks
            .fetch_add(count, Ordering::Relaxed);
    }

    #[cfg(not(test))]
    fn record_callbacks_enqueued(&self, _count: usize) {}

    #[cfg(test)]
    fn record_callbacks_completed(&self, count: usize) {
        self.inner
            .metrics
            .pending_callbacks
            .fetch_sub(count, Ordering::Relaxed);
    }

    #[cfg(not(test))]
    fn record_callbacks_completed(&self, _count: usize) {}

    #[cfg(test)]
    fn metrics_snapshot(&self) -> TaskCallbackDispatcherMetricsSnapshot {
        TaskCallbackDispatcherMetricsSnapshot {
            delivery_runs: self.inner.metrics.delivery_runs.load(Ordering::Acquire),
            delivered_callbacks: self
                .inner
                .metrics
                .delivered_callbacks
                .load(Ordering::Acquire),
            max_callbacks_per_run: self
                .inner
                .metrics
                .max_callbacks_per_run
                .load(Ordering::Acquire),
            pending_callbacks: self.inner.metrics.pending_callbacks.load(Ordering::Acquire),
        }
    }

    #[cfg(test)]
    pub(super) fn pending_callback_count(&self) -> usize {
        self.metrics_snapshot().pending_callbacks
    }
}

impl CallbackEnvelope {
    fn run(&mut self, limit: usize) -> usize {
        let mut callbacks_run = 0;
        while callbacks_run < limit {
            let Some(callback) = self
                .callbacks
                .pop_front()
                .or_else(|| self.completion.take())
            else {
                break;
            };
            let _ = catch_unwind(AssertUnwindSafe(callback));
            callbacks_run += 1;
        }
        callbacks_run
    }

    fn is_complete(&self) -> bool {
        self.callbacks.is_empty() && self.completion.is_none()
    }
}

#[cfg(test)]
mod tests;
