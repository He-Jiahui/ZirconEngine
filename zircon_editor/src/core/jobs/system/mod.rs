mod pending;
mod state;

use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Instant;

use zircon_runtime::core::runtime::tasks::JobScheduler;

use crate::core::editor_message::SharedEditorMessageBus;

use self::pending::PendingJob;
use self::state::EditorJobSystemState;
use super::event_sink::JobEventSink;
use super::pump::JobEventPump;
use super::{
    EditorJob, EditorJobLimits, EditorJobProgressSource, EditorJobSpec, JobContext, JobError,
    JobEventKind, JobId, JobSubmitError, JobTicket, UnfinishedEditorJob,
};

#[derive(Clone)]
pub struct EditorJobSystem {
    inner: Arc<EditorJobSystemInner>,
}

struct EditorJobSystemInner {
    scheduler: JobScheduler,
    limits: EditorJobLimits,
    event_sender: mpsc::Sender<super::JobEvent>,
    event_pump: JobEventPump,
    state: Mutex<EditorJobSystemState>,
    state_changed: Condvar,
    progress: EditorJobProgressSource,
}

impl EditorJobSystem {
    pub fn with_scheduler(scheduler: JobScheduler, limits: EditorJobLimits) -> Self {
        Self::with_scheduler_and_bus(scheduler, SharedEditorMessageBus::default(), limits)
    }

    pub fn with_scheduler_and_bus(
        scheduler: JobScheduler,
        bus: SharedEditorMessageBus,
        limits: EditorJobLimits,
    ) -> Self {
        let limits = limits.with_runtime_defaults(scheduler.parallelism());
        let (event_sender, event_receiver) = mpsc::channel();
        Self {
            inner: Arc::new(EditorJobSystemInner {
                scheduler,
                limits,
                event_sender,
                event_pump: JobEventPump::new(bus, event_receiver),
                state: Mutex::new(EditorJobSystemState::default()),
                state_changed: Condvar::new(),
                progress: EditorJobProgressSource::default(),
            }),
        }
    }

    pub fn submit<J>(
        &self,
        spec: EditorJobSpec,
        job: J,
    ) -> Result<JobTicket<J::Output>, JobSubmitError>
    where
        J: EditorJob,
    {
        if spec.label.trim().is_empty() {
            return Err(JobSubmitError::EmptyLabel);
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancel_sender = sender.clone();
        let cancel_task = Box::new(move |context: JobContext| {
            context.emit(JobEventKind::Cancelled);
            let _ = cancel_sender.send(Err(JobError::Cancelled));
        });
        let task = Box::new(move |context: JobContext| {
            let event_context = context.clone();
            let result = if context.is_cancelled() {
                Err(JobError::Cancelled)
            } else {
                catch_unwind(AssertUnwindSafe(|| job.run(context)))
                    .unwrap_or_else(|payload| Err(JobError::Panicked(panic_message(payload))))
            };
            let kind = match &result {
                Ok(_) => JobEventKind::Completed,
                Err(JobError::Cancelled) => JobEventKind::Cancelled,
                Err(error) => JobEventKind::Failed {
                    message: error.to_string(),
                },
            };
            event_context.emit(kind);
            let _ = sender.send(result);
        });

        let id = {
            let mut state = self.inner.lock_state();
            state.ensure_accepting_submissions()?;
            for dependency in &spec.after {
                state.validate_dependency(*dependency)?;
            }
            let id = state.allocate_id();
            state.register(id);
            self.inner.progress.register(id, &spec);
            state
                .pending
                .push(PendingJob::new(id, spec, task, cancel_task));
            id
        };
        self.inner.promote();
        Ok(JobTicket::new(id, receiver))
    }

    pub fn pump_events(&self) -> usize {
        self.inner.event_pump.pump()
    }

    pub fn progress(&self) -> EditorJobProgressSource {
        self.inner.progress.clone()
    }

    /// Runs two borrowing tasks through the shared runtime scheduler without creating editor threads.
    pub fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.inner.scheduler.join(task_a, task_b)
    }

    /// Cancels a pending job synchronously or requests cooperative cancellation from active work.
    pub fn cancel(&self, id: JobId) -> bool {
        let mut state = self.inner.lock_state();
        let Some(index) = state.pending.iter().position(|pending| pending.id == id) else {
            return self.inner.progress.request_cancel(id);
        };
        let pending = state.pending.remove(index);
        pending.spec.cancel.cancel();
        let label = pending.spec.label.clone();
        let category = pending.spec.category;
        let cancel = pending.spec.cancel.clone();
        let cancel_task = pending.cancel_task;
        drop(state);
        let events = JobEventSink::new(
            id,
            label,
            category,
            self.inner.event_sender.clone(),
            self.inner.progress.clone(),
        );
        cancel_task(JobContext::new(cancel, events));
        let mut state = self.inner.lock_state();
        state.mark_cancelled(id);
        self.inner.progress.complete(id);
        drop(state);
        self.inner.state_changed.notify_all();
        self.inner.promote();
        true
    }

    pub fn shutdown(&self, deadline: Instant) -> Vec<UnfinishedEditorJob> {
        let pending = {
            let mut state = self.inner.lock_state();
            let pending = state.begin_shutdown();
            self.inner.progress.cancel_all();
            pending
        };
        self.inner.cancel_pending(pending);

        let mut state = self.inner.lock_state();
        while self.inner.progress.has_active() {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let wait = deadline.saturating_duration_since(now);
            let (next_state, timeout) = self
                .inner
                .state_changed
                .wait_timeout(state, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if timeout.timed_out() {
                break;
            }
        }
        self.inner.progress.unfinished_jobs()
    }

    #[cfg(test)]
    pub(super) fn retained_record_count(&self) -> usize {
        self.inner.lock_state().retained_record_count()
    }

    #[cfg(test)]
    pub(super) fn is_terminal_record(&self, id: JobId) -> bool {
        self.inner.lock_state().is_terminal_record(id)
    }

    #[cfg(test)]
    pub(super) fn scheduled_record_count(&self) -> usize {
        self.inner.lock_state().scheduled_record_count()
    }

    #[cfg(test)]
    pub(super) fn pending_job_count(&self) -> usize {
        self.inner.lock_state().pending.len()
    }

    #[cfg(test)]
    pub(super) fn running_job_count(&self) -> usize {
        self.inner.lock_state().running_job_count()
    }

    #[cfg(test)]
    pub(super) fn mutex_group_tail_count(&self) -> usize {
        self.inner.lock_state().mutex_group_tail_count()
    }

    #[cfg(test)]
    pub(super) const fn terminal_record_retention_limit(&self) -> usize {
        state::TERMINAL_RECORD_RETENTION_LIMIT
    }
}

impl EditorJobSystemInner {
    fn cancel_pending(&self, pending: Vec<PendingJob>) {
        if pending.is_empty() {
            return;
        }

        let mut cancelled_ids = Vec::with_capacity(pending.len());
        for pending in pending {
            let id = pending.id;
            let events = JobEventSink::new(
                id,
                pending.spec.label,
                pending.spec.category,
                self.event_sender.clone(),
                self.progress.clone(),
            );
            (pending.cancel_task)(JobContext::new(pending.spec.cancel, events));
            cancelled_ids.push(id);
        }
        let mut state = self.lock_state();
        for id in cancelled_ids {
            state.mark_cancelled(id);
            self.progress.complete(id);
        }
        drop(state);
        self.state_changed.notify_all();
    }

    fn promote(self: &Arc<Self>) {
        let mut state = self.lock_state();
        loop {
            let Some(index) = state.next_admissible_index(&self.limits) else {
                break;
            };
            let pending = state.pending.remove(index);
            let explicit_dependencies = pending
                .spec
                .after
                .iter()
                .map(|id| state.dependency_handle(*id))
                .collect::<Option<Vec<_>>>();
            let Some(mut dependencies) = explicit_dependencies else {
                state.pending.push(pending);
                break;
            };
            if let Some(group) = pending.spec.mutex_group.as_ref() {
                if let Some(group_tail) = state.mutex_group_tail(group) {
                    dependencies.push(group_tail);
                }
            }
            state.mark_started(&pending);

            let id = pending.id;
            let category = pending.spec.category;
            let events = JobEventSink::new(
                id,
                pending.spec.label.clone(),
                category,
                self.event_sender.clone(),
                self.progress.clone(),
            );
            let context = JobContext::new(pending.spec.cancel.clone(), events.clone());
            drop(pending.cancel_task);
            let task = pending.task;
            let inner = Arc::clone(self);
            let handle = self.scheduler.schedule_after(&dependencies, move || {
                let _completion = CompletionGuard::new(inner, id, category);
                events.emit(JobEventKind::Started);
                task(context);
            });
            // Completion takes the same state lock, so it cannot overtake this handle install.
            state.store_scheduled_handle(id, handle.clone());
            if let Some(mutex_group) = pending.spec.mutex_group {
                state.update_mutex_group_tail(mutex_group, id, handle);
            }
        }
    }

    fn finish(self: &Arc<Self>, id: JobId, category: super::JobCategory) {
        {
            let mut state = self.lock_state();
            state.mark_finished(id, category);
            self.progress.complete(id);
        }
        self.state_changed.notify_all();
        self.promote();
    }

    fn lock_state(&self) -> MutexGuard<'_, EditorJobSystemState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct CompletionGuard {
    inner: Arc<EditorJobSystemInner>,
    id: JobId,
    category: super::JobCategory,
}

impl CompletionGuard {
    fn new(inner: Arc<EditorJobSystemInner>, id: JobId, category: super::JobCategory) -> Self {
        Self {
            inner,
            id,
            category,
        }
    }
}

impl Drop for CompletionGuard {
    fn drop(&mut self) {
        self.inner.finish(self.id, self.category);
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}
