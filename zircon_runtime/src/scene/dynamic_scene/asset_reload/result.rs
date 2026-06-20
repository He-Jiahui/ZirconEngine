use crate::{
    asset::{AssetEvent, SceneAsset},
    scene::{
        dynamic_scene::{DynamicSceneError, PreparedDynamicSceneSpawn},
        EntityRemap, LevelSystem, World,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadResult {
    event: AssetEvent<SceneAsset>,
    result: Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
}

impl DynamicSceneAssetReloadResult {
    pub fn new(
        event: AssetEvent<SceneAsset>,
        result: Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    ) -> Self {
        Self { event, result }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn result(&self) -> &Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        &self.result
    }

    pub fn into_result(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.result
    }

    pub fn spawn_into(
        self,
        world: &mut World,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        let Self { event, result } = self;
        let prepared = match result {
            Ok(prepared) => prepared,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        let component_type_count = prepared.component_type_count();
        let entity_count = prepared.entity_count();
        let resource_count = prepared.resource_count();

        match prepared.spawn_into(world) {
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

    pub fn spawn_into_level(
        self,
        level: &LevelSystem,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        level.with_world_mut(|world| self.spawn_into(world))
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
}

impl DynamicSceneAssetReloadSupersededTask {
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
