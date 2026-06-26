use std::sync::{Arc, Mutex, MutexGuard};

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

    pub fn take_ready(&self) -> Option<Result<PreparedDynamicSceneSpawn, DynamicSceneError>> {
        if !self.is_ready() {
            return None;
        }
        lock_spawn_result(&self.result).take()
    }

    pub fn wait_ready(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.completion.wait();
        let label = self.descriptor.label;
        let result = lock_spawn_result(&self.result).take();
        result.unwrap_or_else(|| Err(DynamicSceneError::SpawnTaskResultUnavailable { label }))
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
