use std::collections::HashMap;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::core::framework::foundation::{ConfigManagerError, ConfigPersistenceReport};
use crate::core::CoreError;

use super::commit_fence::ConfigCommitFence;
use super::state::ConfigPersistenceState;
use super::writer::ConfigFileWriter;

pub(super) type ConfigSnapshotSource = Arc<dyn Fn() -> HashMap<String, Value> + Send + Sync>;

const WORKER_THREAD_NAME: &str = "zr-config-persist";
pub(super) struct ConfigPersistenceWorker {
    shared: Arc<ConfigPersistenceShared>,
    thread: Mutex<Option<JoinHandle<()>>>,
    shutdown_timeout: Duration,
}

struct ConfigPersistenceShared {
    path: Arc<PathBuf>,
    snapshot: ConfigSnapshotSource,
    writer: Arc<dyn ConfigFileWriter>,
    commit_fence: Arc<ConfigCommitFence>,
    debounce: Duration,
    state: Mutex<ConfigPersistenceState>,
    changed: Condvar,
}

impl ConfigPersistenceWorker {
    pub(super) fn start(
        path: Arc<PathBuf>,
        snapshot: ConfigSnapshotSource,
        writer: Arc<dyn ConfigFileWriter>,
        debounce: Duration,
        shutdown_timeout: Duration,
        commit_fence: Arc<ConfigCommitFence>,
    ) -> Result<Arc<Self>, CoreError> {
        let shared = Arc::new(ConfigPersistenceShared {
            path,
            snapshot,
            writer,
            commit_fence,
            debounce,
            state: Mutex::new(ConfigPersistenceState::default()),
            changed: Condvar::new(),
        });
        let shared_for_thread = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name(WORKER_THREAD_NAME.to_string())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    run_worker(Arc::clone(&shared_for_thread));
                }));
                if result.is_err() {
                    let mut state = shared_for_thread.lock_state();
                    state.last_error = Some("config persistence worker panicked".to_string());
                    state.worker_exited = true;
                    shared_for_thread.changed.notify_all();
                }
            })
            .map_err(|error| {
                CoreError::ThreadSpawn(format!(
                    "config persistence worker for {}: {error}",
                    shared.path.display()
                ))
            })?;
        Ok(Arc::new(Self {
            shared,
            thread: Mutex::new(Some(thread)),
            shutdown_timeout,
        }))
    }

    pub(super) fn request_persistence(&self, changed: bool) {
        let should_notify = self.shared.lock_state().request_persistence(changed);
        if should_notify {
            self.shared.changed.notify_one();
        }
    }

    pub(super) fn flush(&self, timeout: Duration) -> Result<(), ConfigManagerError> {
        let started = Instant::now();
        let mut state = self.shared.lock_state();
        let target_generation = state.request_force_flush();
        if target_generation <= state.persisted_generation {
            return Ok(());
        }
        self.shared.changed.notify_one();

        loop {
            if state.persisted_generation >= target_generation {
                return Ok(());
            }
            if state.attempted_generation >= target_generation
                && !state.work_requested
                && !state.attempt_in_flight
            {
                if let Some(error) = &state.last_error {
                    return Err(persistence_error(&self.shared.path, error.clone()));
                }
            }
            if state.worker_exited {
                return Err(persistence_error(
                    &self.shared.path,
                    state.last_error.clone().unwrap_or_else(|| {
                        "config persistence worker exited before flush".to_string()
                    }),
                ));
            }

            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(flush_timeout_error(&self.shared.path, timeout));
            }
            let (next_state, wait_result) = self
                .shared
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait_result.timed_out() && state.persisted_generation < target_generation {
                return Err(flush_timeout_error(&self.shared.path, timeout));
            }
        }
    }

    pub(super) fn report(&self) -> ConfigPersistenceReport {
        self.shared.lock_state().report()
    }
}

impl fmt::Debug for ConfigPersistenceWorker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigPersistenceWorker")
            .field("path", &self.shared.path)
            .field("report", &self.report())
            .finish()
    }
}

impl Drop for ConfigPersistenceWorker {
    fn drop(&mut self) {
        let started = Instant::now();
        let mut state = self.shared.lock_state();
        state.shutdown_requested = true;
        state.request_force_flush();
        self.shared.changed.notify_one();

        while !state.worker_exited {
            let remaining = self.shutdown_timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                break;
            }
            let (next_state, wait_result) = self
                .shared
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait_result.timed_out() {
                break;
            }
        }
        let worker_exited = state.worker_exited;
        let shutdown_error = (state.dirty_generation > state.persisted_generation)
            .then(|| state.last_error.clone())
            .flatten();
        let commit_in_progress = !worker_exited && self.shared.commit_fence.cancel();
        drop(state);

        if !worker_exited {
            if commit_in_progress {
                tracing::error!(
                    path = %self.shared.path.display(),
                    timeout_ms = %self.shutdown_timeout.as_millis(),
                    "config persistence shutdown timed out during filesystem commit; new activation will fail fast until it exits"
                );
            } else {
                tracing::error!(
                    path = %self.shared.path.display(),
                    timeout_ms = %self.shutdown_timeout.as_millis(),
                    "config persistence shutdown timed out; the pending commit was fenced"
                );
            }
        } else if let Some(error) = shutdown_error {
            tracing::error!(
                path = %self.shared.path.display(),
                error = %error,
                "config persistence shutdown flush failed"
            );
        }

        let thread = self
            .thread
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if worker_exited {
            if let Some(thread) = thread {
                let _ = thread.join();
            }
        }
    }
}

impl ConfigPersistenceShared {
    fn lock_state(&self) -> MutexGuard<'_, ConfigPersistenceState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn run_worker(shared: Arc<ConfigPersistenceShared>) {
    loop {
        let target_generation = match wait_for_attempt(&shared) {
            Some(target_generation) => target_generation,
            None => return,
        };
        let started = Instant::now();
        let snapshot = (shared.snapshot)();
        let serialized = serde_json::to_vec_pretty(&snapshot);
        let serialized_bytes = serialized.as_ref().map_or(0, |bytes| bytes.len());
        let error = match serialized {
            Ok(bytes) => shared
                .writer
                .write(shared.path.as_path(), &bytes, shared.commit_fence.as_ref())
                .err()
                .map(|error| error.to_string()),
            Err(error) => Some(error.to_string()),
        };

        let mut state = shared.lock_state();
        state.complete_attempt(
            target_generation,
            serialized_bytes,
            started.elapsed(),
            error.clone(),
        );
        let exit_after_failure = state.shutdown_requested && error.is_some();
        if exit_after_failure {
            state.worker_exited = true;
        }
        shared.changed.notify_all();
        if exit_after_failure {
            return;
        }
    }
}

fn wait_for_attempt(shared: &ConfigPersistenceShared) -> Option<u64> {
    let mut state = shared.lock_state();
    loop {
        if state.shutdown_requested && state.dirty_generation <= state.persisted_generation {
            state.worker_exited = true;
            shared.changed.notify_all();
            return None;
        }
        if state.work_requested && state.dirty_generation > state.persisted_generation {
            if !state.force_flush && !state.shutdown_requested {
                let due_at = state
                    .last_dirty_at
                    .unwrap_or_else(Instant::now)
                    .checked_add(shared.debounce);
                if let Some(remaining) =
                    due_at.and_then(|due_at| due_at.checked_duration_since(Instant::now()))
                {
                    let (next_state, _) = shared
                        .changed
                        .wait_timeout(state, remaining)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    state = next_state;
                    continue;
                }
            }
            return Some(state.begin_attempt());
        }
        state = shared
            .changed
            .wait(state)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

fn persistence_error(path: &PathBuf, reason: String) -> ConfigManagerError {
    ConfigManagerError::Persistence {
        path: path.to_string_lossy().into_owned(),
        reason,
    }
}

fn flush_timeout_error(path: &PathBuf, timeout: Duration) -> ConfigManagerError {
    ConfigManagerError::FlushTimedOut {
        path: path.to_string_lossy().into_owned(),
        timeout,
    }
}
