use super::super::error::ServiceError;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
};
use tokio::{
    sync::{oneshot, Notify},
    task::JoinHandle,
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::service) struct JobCensus {
    pub active: usize,
    pub finished: u64,
    pub failed: u64,
}

struct JobState {
    accepting: bool,
    next_id: u64,
    active: BTreeMap<u64, Option<JoinHandle<()>>>,
    finished: u64,
    failed: u64,
}

pub(super) struct JobRegistry {
    state: Mutex<JobState>,
    changed: Notify,
}

impl JobRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(JobState {
                accepting: true,
                next_id: 0,
                active: BTreeMap::new(),
                finished: 0,
                failed: 0,
            }),
            changed: Notify::new(),
        })
    }

    pub fn submit<T, F>(
        self: &Arc<Self>,
        operation: F,
    ) -> Result<oneshot::Receiver<Result<T, ServiceError>>, ServiceError>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, ServiceError> + Send + 'static,
    {
        let job_id = self.reserve()?;
        let (sender, receiver) = oneshot::channel();
        let blocking = tokio::task::spawn_blocking(operation);
        let registry = Arc::clone(self);
        let owner = tokio::spawn(async move {
            let result = match blocking.await {
                Ok(result) => result,
                Err(_) => Err(ServiceError::Storage),
            };
            let failed = result.is_err();
            let _ = sender.send(result);
            registry.finish(job_id, failed);
        });
        self.attach(job_id, owner);
        Ok(receiver)
    }

    pub fn stop_admission(&self) -> JobCensus {
        let mut state = self.state();
        state.accepting = false;
        Self::census_from(&state)
    }

    pub fn census(&self) -> JobCensus {
        Self::census_from(&self.state())
    }

    pub async fn shutdown(&self, deadline: Instant) -> Result<JobCensus, ServiceError> {
        self.stop_admission();
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let census = self.census();
            if census.active == 0 {
                return Ok(census);
            }
            if Instant::now() >= deadline {
                return Err(ServiceError::OutcomeUnknown);
            }
            if tokio::time::timeout_at(deadline, changed).await.is_err() {
                return Err(ServiceError::OutcomeUnknown);
            }
        }
    }

    fn reserve(&self) -> Result<u64, ServiceError> {
        let mut state = self.state();
        if !state.accepting {
            return Err(ServiceError::Capacity);
        }
        let job_id = state.next_id;
        state.next_id = state.next_id.checked_add(1).ok_or(ServiceError::Storage)?;
        state.active.insert(job_id, None);
        Ok(job_id)
    }

    fn attach(&self, job_id: u64, owner: JoinHandle<()>) {
        if let Some(slot) = self.state().active.get_mut(&job_id) {
            *slot = Some(owner);
        }
    }

    fn finish(&self, job_id: u64, failed: bool) {
        let mut state = self.state();
        if state.active.remove(&job_id).is_some() {
            state.finished += 1;
            if failed {
                state.failed += 1;
            }
        }
        drop(state);
        self.changed.notify_waiters();
    }

    fn census_from(state: &JobState) -> JobCensus {
        JobCensus {
            active: state.active.len(),
            finished: state.finished,
            failed: state.failed,
        }
    }

    fn state(&self) -> MutexGuard<'_, JobState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
