use std::sync::{Arc, Mutex};

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
        let mut status = self
            .status
            .lock()
            .expect("dynamic scene spawn task status lock poisoned");
        status.record_poll();
        status.clone()
    }

    pub fn status_snapshot(&self) -> AsyncTaskStatus {
        self.status
            .lock()
            .expect("dynamic scene spawn task status lock poisoned")
            .clone()
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
        self.result
            .lock()
            .expect("dynamic scene spawn task result lock poisoned")
            .take()
    }

    pub fn wait_ready(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.completion.wait();
        let label = self.descriptor.label;
        let result = self
            .result
            .lock()
            .expect("dynamic scene spawn task result lock poisoned")
            .take();
        result.unwrap_or_else(|| Err(DynamicSceneError::SpawnTaskResultUnavailable { label }))
    }
}
