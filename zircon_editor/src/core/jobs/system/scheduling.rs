use std::sync::{Arc, MutexGuard};

use super::pending::PendingJob;
use super::state::EditorJobSystemState;
use super::{EditorJobSystemInner, ProgressObserverEvent};
use crate::core::jobs::event_sink::JobEventSink;
use crate::core::jobs::{JobCategory, JobContext, JobEventKind, JobId};

const MAX_PROMOTION_DISPATCH_BATCH: usize = 64;

impl EditorJobSystemInner {
    pub(super) fn cancel_pending(&self, pending: Vec<PendingJob>) {
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
                self.event_queue.clone(),
                self.progress.clone(),
            );
            (pending.cancel_task)(JobContext::new(pending.spec.cancel, events));
            cancelled_ids.push(id);
        }
        let mut state = self.lock_state();
        for id in &cancelled_ids {
            state.mark_cancelled(*id);
            self.progress.complete(*id);
            self.enqueue_progress_observer_event(ProgressObserverEvent::Finished(*id));
        }
        drop(state);
        self.deliver_progress_observer_events();
        self.state_changed.notify_all();
    }

    pub(super) fn promote(self: &Arc<Self>) {
        // This gate preserves mutex-group chain construction while runtime
        // scheduling remains outside the state mutex.
        let _promotion = self
            .promotion
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for _ in 0..MAX_PROMOTION_DISPATCH_BATCH {
            let dispatch = {
                let mut state = self.lock_state();
                let Some(pending) = state.take_next_admissible(&self.limits) else {
                    break;
                };
                let mut dependencies = pending
                    .spec
                    .after
                    .iter()
                    .map(|id| {
                        state
                            .dependency_handle(*id)
                            .expect("pending dependency records stay pinned until scheduling")
                    })
                    .collect::<Vec<_>>();
                if let Some(group) = pending.spec.mutex_group.as_ref() {
                    if let Some(group_tail) = state.mutex_group_tail(group) {
                        dependencies.push(group_tail);
                    }
                }
                state.mark_started(&pending);
                (pending, dependencies)
            };
            let (pending, dependencies) = dispatch;
            let id = pending.id;
            let category = pending.spec.category;
            let mutex_group = pending.spec.mutex_group.clone();
            let events = JobEventSink::new(
                id,
                pending.spec.label.clone(),
                category,
                self.event_queue.clone(),
                self.progress.clone(),
            );
            let context = JobContext::new(pending.spec.cancel.clone(), events.clone());
            drop(pending.cancel_task);
            let task = pending.task;
            let inner = Arc::clone(self);
            let handle = self.scheduler.schedule_after(&dependencies, move || {
                let _completion = CompletionGuard::new(inner, id, category);
                events.emit(JobEventKind::Started);
                task.run(context);
            });
            let mut state = self.lock_state();
            // Completion can win the race to this lock after `schedule_after`
            // returns. The state helpers preserve a terminal record instead
            // of reinstalling its handle or mutex tail.
            state.store_scheduled_handle(id, handle.clone());
            if let Some(mutex_group) = mutex_group {
                state.update_mutex_group_tail(mutex_group, id, handle);
            }
        }
    }

    pub(super) fn finish(self: &Arc<Self>, id: JobId, category: JobCategory) {
        {
            let mut state = self.lock_state();
            state.mark_finished(id, category);
            self.progress.complete(id);
            self.enqueue_progress_observer_event(ProgressObserverEvent::Finished(id));
        }
        self.deliver_progress_observer_events();
        self.state_changed.notify_all();
        self.promote();
    }

    pub(super) fn lock_state(&self) -> MutexGuard<'_, EditorJobSystemState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct CompletionGuard {
    inner: Arc<EditorJobSystemInner>,
    id: JobId,
    category: JobCategory,
}

impl CompletionGuard {
    fn new(inner: Arc<EditorJobSystemInner>, id: JobId, category: JobCategory) -> Self {
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
