use crate::{
    asset::{AssetEvent, SceneAsset},
    core::framework::tasks::{AsyncTaskDescriptor, AsyncTaskState, AsyncTaskStatus},
    scene::dynamic_scene::DynamicSceneSpawnTask,
};

#[derive(Debug)]
pub struct DynamicSceneAssetReloadTask {
    pub(super) event: AssetEvent<SceneAsset>,
    pub(super) task: DynamicSceneSpawnTask,
}

impl DynamicSceneAssetReloadTask {
    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn task(&self) -> &DynamicSceneSpawnTask {
        &self.task
    }

    pub fn is_ready(&self) -> bool {
        self.task.is_ready()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadPendingTaskSnapshot {
    event: AssetEvent<SceneAsset>,
    descriptor: AsyncTaskDescriptor,
    status: AsyncTaskStatus,
}

impl DynamicSceneAssetReloadPendingTaskSnapshot {
    pub fn new(
        event: AssetEvent<SceneAsset>,
        descriptor: AsyncTaskDescriptor,
        status: AsyncTaskStatus,
    ) -> Self {
        Self {
            event,
            descriptor,
            status,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn descriptor(&self) -> &AsyncTaskDescriptor {
        &self.descriptor
    }

    pub fn status(&self) -> &AsyncTaskStatus {
        &self.status
    }

    pub fn state(&self) -> AsyncTaskState {
        self.status.state
    }

    pub fn is_collectable(&self) -> bool {
        self.status.is_terminal()
    }
}
