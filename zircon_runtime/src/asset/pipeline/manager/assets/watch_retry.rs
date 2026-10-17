//! One cancellable delayed retry per watcher activation. No asset pool thread sleeps here.
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::project_asset_manager::{ProjectWatcherActivation, ProjectWatcherLifecycle};
use super::runtime::WatchRetryWork;
use super::watch_dispatch::{merge_batch, should_schedule_worker};
use super::ProjectAssetManager;

#[derive(Default)]
struct RetrySignal { cancelled: bool }

pub(in crate::asset::pipeline::manager) struct WatchRetryTimer {
    signal: Arc<(Mutex<RetrySignal>, Condvar)>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl WatchRetryTimer {
    fn start(
        activation: Weak<ProjectWatcherActivation>,
        manager: ProjectAssetManager,
        ingress: Arc<()>,
    ) -> std::io::Result<Arc<Self>> {
        let signal = Arc::new((Mutex::new(RetrySignal::default()), Condvar::new()));
        let worker_signal = Arc::clone(&signal);
        let handle = thread::Builder::new().name("asset-watch-retry".into()).spawn(move || {
            let (lock, wake) = &*worker_signal;
            let state = lock.lock().unwrap_or_else(|p| p.into_inner());
            let (state, _) = wake.wait_timeout_while(state, Duration::from_millis(25), |state| !state.cancelled)
                .unwrap_or_else(|p| p.into_inner());
            let cancelled = state.cancelled;
            drop(state);
            if !cancelled {
                if let Some(activation) = activation.upgrade() {
                    activation.wake_watch_retry(&manager, &worker_signal, &ingress);
                }
            }
        })?;
        Ok(Arc::new(Self { signal, handle: Mutex::new(Some(handle)) }))
    }

    pub(super) fn cancel(&self) {
        let (lock, wake) = &*self.signal;
        lock.lock().unwrap_or_else(|p| p.into_inner()).cancelled = true;
        wake.notify_all();
    }

    /// Called outside the activation lock. Deadline retirement never detaches an unfinished handle.
    pub(super) fn join(&self, blocking: bool) -> bool {
        let mut handle = if blocking {
            self.handle.lock().unwrap_or_else(|p| p.into_inner())
        } else {
            match self.handle.try_lock() {
                Ok(handle) => handle,
                Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => return false,
            }
        };
        let Some(thread) = handle.as_ref() else { return true; };
        if thread.thread().id() == thread::current().id() { return false; }
        if !blocking && !thread.is_finished() { return false; }
        let _ = handle.take().expect("owned retry thread").join();
        true
    }
}

impl Drop for WatchRetryTimer {
    fn drop(&mut self) {
        self.cancel();
        let handle = self.handle.get_mut().unwrap_or_else(|p| p.into_inner()).take();
        if let Some(handle) = handle {
            if handle.thread().id() != thread::current().id() {
                let _ = handle.join();
            }
            // Last activation Arc may be released by this timer. Its own thread exits naturally.
        }
    }
}

impl ProjectWatcherActivation {
    pub(super) fn defer_watch_work(self: &Arc<Self>, manager: &ProjectAssetManager, work: WatchRetryWork) {
        let ingress = Arc::clone(&self.lock_state().ingress_epoch);
        self.defer_watch_work_at(manager, work, ingress);
    }

    pub(super) fn defer_watch_work_at(
        self: &Arc<Self>, manager: &ProjectAssetManager, mut work: WatchRetryWork, ingress: Arc<()>,
    ) {
        // Reap before making another thread; never overlap two timer threads for one activation.
        let previous_timer = self.lock_state().retry_timer.clone();
        if let Some(timer) = &previous_timer {
            timer.cancel();
            if !timer.join(true) { return; }
        }
        let mut state = self.lock_state();
        if let Some(previous) = &previous_timer {
            if state.retry_timer.as_ref().is_some_and(|timer| Arc::ptr_eq(timer, previous)) {
                state.retry_timer = None;
            }
        }
        if state.lifecycle == ProjectWatcherLifecycle::Retired { drop(state); drop(work); return; }
        if !Arc::ptr_eq(&state.ingress_epoch, &ingress) {
            // New input supersedes the owned importer output, but keeps its event responsibility.
            let batch = work.take_batch();
            merge_batch(&mut state, batch);
            let schedule = should_schedule_worker(&mut state);
            drop(state);
            drop(work);
            if schedule { self.spawn_worker(manager.clone()); }
            return;
        }
        let abandoned = state.retry_work.replace(work);
        state.retry_ready = false;
        match WatchRetryTimer::start(Arc::downgrade(self), manager.clone(), ingress) {
            Ok(timer) => state.retry_timer = Some(timer),
            Err(error) => {
                // Keep the pending responsibility visible; do not convert it to an importer Error.
                drop(state);
                drop(abandoned);
                manager.broadcast_watch_error(crate::asset::watch::AssetWatchError::from_message(
                    std::path::PathBuf::from("asset-watch-retry"), error.to_string(),
                ));
                return;
            }
        }
        drop(state);
        drop(abandoned);
    }

    fn wake_watch_retry(
        self: &Arc<Self>,
        manager: &ProjectAssetManager,
        token: &Arc<(Mutex<RetrySignal>, Condvar)>,
        ingress: &Arc<()>,
    ) {
        let schedule = {
            let mut state = self.lock_state();
            let current = state.retry_timer.as_ref().is_some_and(|timer| Arc::ptr_eq(&timer.signal, token));
            if state.lifecycle != ProjectWatcherLifecycle::Active || !current
                || !Arc::ptr_eq(&state.ingress_epoch, ingress) || state.retry_work.is_none()
            { return; }
            state.retry_ready = true;
            should_schedule_worker(&mut state)
        };
        if schedule { self.spawn_worker(manager.clone()); }
    }

    pub(super) fn retire_retry(&self, deadline: Option<Instant>) -> bool {
        let (timer, work) = {
            let mut state = if deadline.is_some() {
                match self.state.try_lock() {
                    Ok(state) => state,
                    Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner(),
                    Err(std::sync::TryLockError::WouldBlock) => return false,
                }
            } else { self.lock_state() };
            Self::retire_locked(&mut state);
            state.ingress_epoch = Arc::new(());
            state.retry_ready = false;
            (state.retry_timer.clone(), state.retry_work.take())
        };
        drop(work);
        let Some(timer) = timer else { return true; };
        timer.cancel();
        let joined = timer.join(deadline.is_none());
        if joined {
            let mut state = if deadline.is_some() {
                match self.state.try_lock() {
                    Ok(state) => state,
                    Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner(),
                    Err(std::sync::TryLockError::WouldBlock) => return false,
                }
            } else { self.lock_state() };
            if state.retry_timer.as_ref().is_some_and(|current| Arc::ptr_eq(current, &timer)) {
                state.retry_timer = None;
            }
        }
        joined
    }
}

impl Drop for ProjectWatcherActivation {
    fn drop(&mut self) { self.retire_retry(None); }
}
