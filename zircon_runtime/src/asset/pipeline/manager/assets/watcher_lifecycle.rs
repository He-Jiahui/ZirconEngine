use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use crate::core::CoreError;

use super::super::errors::asset_error_message;
use super::ProjectAssetManager;

#[derive(Default)]
pub(super) struct ProjectWatcherAdmission {
    closing: AtomicBool,
    state: Mutex<AdmissionState>,
}

#[derive(Default)]
struct AdmissionState {
    operations: usize,
}

pub(super) struct ProjectWatcherOperation<'a> {
    admission: &'a ProjectWatcherAdmission,
}

pub(super) struct ProjectWatcherPublication<'a> {
    _state: MutexGuard<'a, AdmissionState>,
}

impl ProjectWatcherAdmission {
    fn lock(&self) -> MutexGuard<'_, AdmissionState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub(super) fn try_close(&self) -> bool {
        // Closing admission must survive contention with a publication already in progress.
        self.closing.store(true, Ordering::Release);
        let state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return false,
        };
        state.operations == 0
    }
}

impl ProjectAssetManager {
    pub(super) fn begin_project_watcher_operation(
        &self,
    ) -> Result<ProjectWatcherOperation<'_>, CoreError> {
        let mut state = self.watcher_admission.lock();
        if self.watcher_admission.closing.load(Ordering::Acquire) {
            return Err(asset_error_message("project watcher admission is closed"));
        }
        state.operations = state.operations.checked_add(1).ok_or_else(|| {
            asset_error_message("project watcher operation count exceeded its limit")
        })?;
        Ok(ProjectWatcherOperation {
            admission: &self.watcher_admission,
        })
    }

    pub(super) fn admit_project_watcher_publication(
        &self,
    ) -> Result<ProjectWatcherPublication<'_>, CoreError> {
        let state = self.watcher_admission.lock();
        if self.watcher_admission.closing.load(Ordering::Acquire) {
            return Err(asset_error_message("project watcher admission is closed"));
        }
        Ok(ProjectWatcherPublication { _state: state })
    }
}

impl Drop for ProjectWatcherOperation<'_> {
    fn drop(&mut self) {
        let mut state = self.admission.lock();
        state.operations -= 1;
    }
}
