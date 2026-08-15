use std::any::Any;
use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

use super::EditorJobSystemInner;
use crate::core::jobs::JobId;

const MAX_PROGRESS_OBSERVER_EVENTS: usize = 1_024;

#[derive(Clone, Copy)]
pub(super) enum ProgressObserverEvent {
    Admitted(JobId),
    Finished(JobId),
    Resynchronize,
}

#[derive(Default)]
pub(super) struct ProgressObserverDispatch {
    events: VecDeque<ProgressObserverEvent>,
    delivering: bool,
    resynchronize_queued: bool,
}

impl ProgressObserverDispatch {
    fn push(&mut self, event: ProgressObserverEvent) {
        if self.resynchronize_queued {
            return;
        }
        if self.events.len() >= MAX_PROGRESS_OBSERVER_EVENTS {
            self.request_resynchronize();
            return;
        }
        self.events.push_back(event);
    }

    fn request_resynchronize(&mut self) {
        self.events.clear();
        self.events.push_back(ProgressObserverEvent::Resynchronize);
        self.resynchronize_queued = true;
    }

    fn pop_front(&mut self) -> Option<ProgressObserverEvent> {
        let event = self.events.pop_front()?;
        if matches!(event, ProgressObserverEvent::Resynchronize) {
            self.resynchronize_queued = false;
        }
        Some(event)
    }
}

impl EditorJobSystemInner {
    pub(super) fn enqueue_progress_observer_event(&self, event: ProgressObserverEvent) {
        if self.progress_observer.is_none() {
            return;
        }
        self.progress_observer_dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event);
    }

    pub(super) fn deliver_progress_observer_events(&self) {
        let Some(observer) = &self.progress_observer else {
            return;
        };
        {
            let mut dispatch = self
                .progress_observer_dispatch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if dispatch.delivering {
                return;
            }
            dispatch.delivering = true;
        }
        let mut delivery_guard = ProgressObserverDeliveryGuard {
            dispatch: &self.progress_observer_dispatch,
            armed: true,
        };
        loop {
            let event = {
                let mut dispatch = self
                    .progress_observer_dispatch
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let Some(event) = dispatch.pop_front() else {
                    dispatch.delivering = false;
                    delivery_guard.disarm();
                    return;
                };
                event
            };
            let callback_result = catch_unwind(AssertUnwindSafe(|| match event {
                ProgressObserverEvent::Admitted(id) => observer.job_admitted(id, &self.progress),
                ProgressObserverEvent::Finished(id) => observer.job_finished(id, &self.progress),
                ProgressObserverEvent::Resynchronize => {
                    observer.jobs_resynchronized(&self.progress)
                }
            }));
            if let Err(payload) = callback_result {
                tracing::error!(
                    panic = %panic_message(payload),
                    "editor job progress observer callback panicked"
                );
                let recovered = catch_unwind(AssertUnwindSafe(|| {
                    observer.jobs_resynchronized(&self.progress)
                }));
                if let Err(payload) = recovered {
                    tracing::error!(
                        panic = %panic_message(payload),
                        "editor job progress observer resynchronization panicked"
                    );
                    let mut dispatch = self
                        .progress_observer_dispatch
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    dispatch.request_resynchronize();
                    dispatch.delivering = false;
                    delivery_guard.disarm();
                    return;
                }
            }
        }
    }
}

struct ProgressObserverDeliveryGuard<'a> {
    dispatch: &'a Mutex<ProgressObserverDispatch>,
    armed: bool,
}

impl ProgressObserverDeliveryGuard<'_> {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ProgressObserverDeliveryGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.dispatch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .delivering = false;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_event_backlog_collapses_to_one_authoritative_resynchronization() {
        let mut dispatch = ProgressObserverDispatch::default();
        for id in 0..=MAX_PROGRESS_OBSERVER_EVENTS {
            dispatch.push(ProgressObserverEvent::Admitted(JobId::new(id as u64)));
        }

        assert_eq!(dispatch.events.len(), 1);
        assert!(matches!(
            dispatch.events.front(),
            Some(ProgressObserverEvent::Resynchronize)
        ));

        dispatch.push(ProgressObserverEvent::Finished(JobId::new(1)));
        assert_eq!(dispatch.events.len(), 1);
        assert!(matches!(
            dispatch.pop_front(),
            Some(ProgressObserverEvent::Resynchronize)
        ));

        dispatch.push(ProgressObserverEvent::Finished(JobId::new(2)));
        assert_eq!(dispatch.events.len(), 1);
    }
}
