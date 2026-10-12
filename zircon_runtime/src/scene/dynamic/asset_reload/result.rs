use std::time::{Duration, Instant};

use crate::{
    asset::{AssetEvent, SceneAsset},
    scene::{
        dynamic_scene::{DynamicSceneError, PreparedDynamicSceneSpawn},
        EntityRemap,
    },
};

#[cfg(test)]
use crate::scene::{LevelSystem, World};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DynamicSceneAssetReloadResult {
    event: AssetEvent<SceneAsset>,
    result: Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    estimated_bytes: usize,
    queued_at: Instant,
}

impl DynamicSceneAssetReloadResult {
    pub(crate) fn new(
        event: AssetEvent<SceneAsset>,
        result: Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    ) -> Self {
        let estimated_bytes = estimate_result_bytes(&result);
        Self {
            event,
            result,
            estimated_bytes,
            queued_at: Instant::now(),
        }
    }

    pub(crate) fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub(crate) fn result(&self) -> &Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        &self.result
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        AssetEvent<SceneAsset>,
        Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    ) {
        (self.event, self.result)
    }

    pub(crate) fn into_result(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.result
    }

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    pub(crate) fn age(&self) -> Duration {
        self.queued_at.elapsed()
    }

    pub(crate) fn bounded_to(self, limit_bytes: usize) -> Self {
        let estimated_bytes = self.estimated_bytes();
        if estimated_bytes <= limit_bytes {
            return self;
        }
        let result = Err(DynamicSceneError::ReloadResultTooLarge {
            estimated_bytes,
            limit_bytes,
        });
        Self {
            event: self.event,
            estimated_bytes: estimate_result_bytes(&result),
            result,
            queued_at: self.queued_at,
        }
    }

    #[cfg(test)]
    pub(crate) fn spawn_into(
        self,
        world: &mut World,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        self.spawn_into_with_target_limit(world, usize::MAX)
    }

    #[cfg(test)]
    pub(crate) fn spawn_into_with_target_limit(
        self,
        world: &mut World,
        target_snapshot_limit_bytes: usize,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        let Self { event, result, .. } = self;
        let prepared = match result {
            Ok(prepared) => prepared,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        let component_type_count = prepared.component_type_count();
        let entity_count = prepared.entity_count();
        let resource_count = prepared.resource_count();

        let staged = match prepared.stage_into_with_limit(world, target_snapshot_limit_bytes) {
            Ok(staged) => staged,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        match staged.commit_into(world) {
            Ok(remap) => Ok(DynamicSceneAssetReloadAppliedScene::new(
                event,
                remap,
                component_type_count,
                entity_count,
                resource_count,
            )),
            Err(error) => Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        }
    }

    #[cfg(test)]
    pub(crate) fn spawn_into_level(
        self,
        level: &LevelSystem,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        self.spawn_into_level_with_target_limit(level, usize::MAX)
    }

    #[cfg(test)]
    pub(crate) fn spawn_into_level_with_target_limit(
        self,
        level: &LevelSystem,
        target_snapshot_limit_bytes: usize,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        let Self { event, result, .. } = self;
        let prepared = match result {
            Ok(prepared) => prepared,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        let component_type_count = prepared.component_type_count();
        let entity_count = prepared.entity_count();
        let resource_count = prepared.resource_count();
        let staged = match prepared.stage_into_level(level, target_snapshot_limit_bytes) {
            Ok(staged) => staged,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        match staged.commit_into_level(level) {
            Ok(remap) => Ok(DynamicSceneAssetReloadAppliedScene::new(
                event,
                remap,
                component_type_count,
                entity_count,
                resource_count,
            )),
            Err(error) => Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        }
    }
}

fn estimate_result_bytes(result: &Result<PreparedDynamicSceneSpawn, DynamicSceneError>) -> usize {
    match result {
        Ok(prepared) => prepared.estimated_bytes(),
        Err(error) => std::mem::size_of::<DynamicSceneAssetReloadResult>()
            .saturating_add(error.to_string().len()),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadAppliedScene {
    event: AssetEvent<SceneAsset>,
    remap: EntityRemap,
    component_type_count: usize,
    entity_count: usize,
    resource_count: usize,
}

impl DynamicSceneAssetReloadAppliedScene {
    pub fn new(
        event: AssetEvent<SceneAsset>,
        remap: EntityRemap,
        component_type_count: usize,
        entity_count: usize,
        resource_count: usize,
    ) -> Self {
        Self {
            event,
            remap,
            component_type_count,
            entity_count,
            resource_count,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn remap(&self) -> &EntityRemap {
        &self.remap
    }

    pub fn component_type_count(&self) -> usize {
        self.component_type_count
    }

    pub fn entity_count(&self) -> usize {
        self.entity_count
    }

    pub fn resource_count(&self) -> usize {
        self.resource_count
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadApplyFailure {
    event: AssetEvent<SceneAsset>,
    error: DynamicSceneError,
}

impl DynamicSceneAssetReloadApplyFailure {
    pub fn new(event: AssetEvent<SceneAsset>, error: DynamicSceneError) -> Self {
        Self { event, error }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn error(&self) -> &DynamicSceneError {
        &self.error
    }

    pub fn into_error(self) -> DynamicSceneError {
        self.error
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadStaleResult {
    event: AssetEvent<SceneAsset>,
    latest_revision: u64,
}

impl DynamicSceneAssetReloadStaleResult {
    pub fn new(event: AssetEvent<SceneAsset>, latest_revision: u64) -> Self {
        Self {
            event,
            latest_revision,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn latest_revision(&self) -> u64 {
        self.latest_revision
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadSupersededTask {
    event: AssetEvent<SceneAsset>,
    latest_revision: u64,
    cancellation_requested: bool,
    previous_state: crate::core::TaskState,
}

impl DynamicSceneAssetReloadSupersededTask {
    pub fn new(
        event: AssetEvent<SceneAsset>,
        latest_revision: u64,
        cancellation_requested: bool,
        previous_state: crate::core::TaskState,
    ) -> Self {
        Self {
            event,
            latest_revision,
            cancellation_requested,
            previous_state,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn latest_revision(&self) -> u64 {
        self.latest_revision
    }

    pub fn cancellation_requested(&self) -> bool {
        self.cancellation_requested
    }

    pub fn previous_state(&self) -> crate::core::TaskState {
        self.previous_state
    }
}

#[cfg(test)]
#[path = "tests/result.rs"]
mod tests;
