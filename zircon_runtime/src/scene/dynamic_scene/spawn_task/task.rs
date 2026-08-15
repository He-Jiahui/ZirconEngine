use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};

use crate::core::framework::tasks::{AsyncTaskDescriptor, AsyncTaskStatus};
use crate::core::JobHandle;

use super::super::DynamicSceneError;
use super::prepared::PreparedDynamicSceneSpawn;
use super::SpawnTaskResult;

/// Background scene-load/preparation task; world mutation stays on the caller thread.
#[derive(Debug)]
pub struct DynamicSceneSpawnTask {
    pub(super) descriptor: AsyncTaskDescriptor,
    pub(super) status: Arc<Mutex<AsyncTaskStatus>>,
    pub(super) completion: JobHandle,
    pub(super) result: Arc<Mutex<Option<SpawnTaskResult>>>,
    pub(super) cancel_requested: Arc<AtomicBool>,
}

impl DynamicSceneSpawnTask {
    pub fn descriptor(&self) -> &AsyncTaskDescriptor {
        &self.descriptor
    }

    pub fn status(&self) -> AsyncTaskStatus {
        let mut status = lock_spawn_status(&self.status);
        status.record_poll();
        status.clone()
    }

    pub fn status_snapshot(&self) -> AsyncTaskStatus {
        lock_spawn_status(&self.status).clone()
    }

    pub fn completion_handle(&self) -> JobHandle {
        self.completion.clone()
    }

    pub fn is_ready(&self) -> bool {
        self.completion.is_complete()
    }

    pub fn request_cancel(&self) -> bool {
        let status = lock_spawn_status(&self.status);
        if status.is_terminal() {
            return false;
        }
        if self.cancel_requested.swap(true, Ordering::AcqRel) {
            return false;
        }
        lock_spawn_result(&self.result).take();
        true
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.cancel_requested.load(Ordering::Acquire)
    }

    pub(crate) fn ready_estimated_bytes(&self) -> Option<usize> {
        if !self.is_ready() {
            return None;
        }
        lock_spawn_result(&self.result).as_ref().map(|result| {
            result
                .as_ref()
                .map_or(0, |prepared| prepared.estimated_bytes())
        })
    }

    pub fn take_ready(&self) -> Option<Result<PreparedDynamicSceneSpawn, DynamicSceneError>> {
        if !self.is_ready() {
            return None;
        }
        if let Some(result) = lock_spawn_result(&self.result).take() {
            return Some(result);
        }
        let label = self.descriptor.label.clone();
        Some(if self.is_cancellation_requested() {
            Err(DynamicSceneError::SpawnTaskCancelled { label })
        } else {
            Err(DynamicSceneError::SpawnTaskResultUnavailable { label })
        })
    }

    pub fn wait_ready(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.completion.wait();
        let label = self.descriptor.label.clone();
        let result = lock_spawn_result(&self.result).take();
        result.unwrap_or_else(|| {
            if self.is_cancellation_requested() {
                Err(DynamicSceneError::SpawnTaskCancelled { label })
            } else {
                Err(DynamicSceneError::SpawnTaskResultUnavailable { label })
            }
        })
    }
}

impl Drop for DynamicSceneSpawnTask {
    fn drop(&mut self) {
        self.request_cancel();
    }
}

pub(super) fn lock_spawn_status(
    status: &Mutex<AsyncTaskStatus>,
) -> MutexGuard<'_, AsyncTaskStatus> {
    status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn lock_spawn_result(
    result: &Mutex<Option<SpawnTaskResult>>,
) -> MutexGuard<'_, Option<SpawnTaskResult>> {
    result
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::core::framework::tasks::{AsyncTaskHandle, AsyncTaskState};

    use super::*;

    #[test]
    fn dynamic_scene_spawn_task_accessors_recover_poisoned_locks() {
        let handle = AsyncTaskHandle::new(17);
        let status = Arc::new(Mutex::new(AsyncTaskStatus::pending(handle)));
        let result = Arc::new(Mutex::new(Some(Err(DynamicSceneError::Parse {
            reason: "decode failed".to_string(),
        }))));

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = status.lock().unwrap();
            panic!("poison dynamic scene spawn task status lock");
        }));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = result.lock().unwrap();
            panic!("poison dynamic scene spawn task result lock");
        }));

        {
            let mut status = lock_spawn_status(&status);
            status.record_poll();
            status.mark_running();
        }
        let status_snapshot = lock_spawn_status(&status).clone();
        assert_eq!(status_snapshot.poll_count, 1);
        assert_eq!(status_snapshot.state, AsyncTaskState::Running);

        let recovered = lock_spawn_result(&result)
            .take()
            .expect("result should remain available after poison recovery");
        assert!(matches!(recovered, Err(DynamicSceneError::Parse { .. })));
    }
}
