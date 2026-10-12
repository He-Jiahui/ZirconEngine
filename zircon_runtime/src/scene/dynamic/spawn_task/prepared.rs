use std::io::{self, Write};

use crate::{
    core::framework::scene::WorldHandle,
    scene::{EntityRemap, LevelSystem, World},
};

use super::super::{
    scene::{CompiledSceneSpawn, PreflightedSceneMutation},
    DynamicScene, DynamicSceneError,
};

/// A validated dynamic scene payload that is ready to apply on the main world.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedDynamicSceneSpawn {
    scene: DynamicScene,
    component_type_count: usize,
    entity_count: usize,
    resource_count: usize,
    estimated_bytes: usize,
}

impl PreparedDynamicSceneSpawn {
    pub fn new(scene: DynamicScene) -> Result<Self, DynamicSceneError> {
        Self::new_with_limit(scene, usize::MAX)
    }

    pub(crate) fn new_with_limit(
        scene: DynamicScene,
        limit_bytes: usize,
    ) -> Result<Self, DynamicSceneError> {
        scene.ensure_supported()?;
        let estimated_bytes = estimate_scene_bytes(&scene)?;
        if estimated_bytes > limit_bytes {
            return Err(DynamicSceneError::PreparedPayloadTooLarge {
                estimated_bytes,
                limit_bytes,
            });
        }
        Ok(Self {
            component_type_count: scene.component_types.len(),
            entity_count: scene.entities.len(),
            resource_count: scene.resources.len(),
            estimated_bytes,
            scene,
        })
    }

    pub fn scene(&self) -> &DynamicScene {
        &self.scene
    }

    pub fn into_scene(self) -> DynamicScene {
        self.scene
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

    pub fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    pub fn spawn_into(self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        self.scene.spawn_into(world)
    }

    pub(crate) fn stage_into(
        self,
        world: &mut World,
    ) -> Result<StagedDynamicSceneSpawn, DynamicSceneError> {
        self.stage_into_with_limit(world, usize::MAX)
    }

    pub(crate) fn stage_into_with_limit(
        self,
        world: &mut World,
        target_snapshot_limit_bytes: usize,
    ) -> Result<StagedDynamicSceneSpawn, DynamicSceneError> {
        let target = self.capture_world_target(world, target_snapshot_limit_bytes)?;
        self.stage_target(target)
    }

    pub(crate) fn capture_world_target(
        &self,
        world: &mut World,
        target_snapshot_limit_bytes: usize,
    ) -> Result<DynamicSceneSpawnTargetSnapshot, DynamicSceneError> {
        let expected_generation = world.world_generation();
        let plan = self.scene.compile_spawn_into(world)?;
        let (preflight_world, estimated_bytes) = DynamicScene::capture_compiled_spawn_preflight(
            world,
            &plan,
            target_snapshot_limit_bytes,
        )?;
        Ok(DynamicSceneSpawnTargetSnapshot {
            expected_generation,
            target_level: None,
            preflight_world,
            plan,
            estimated_bytes,
        })
    }

    pub(crate) fn stage_into_level(
        self,
        level: &LevelSystem,
        target_snapshot_limit_bytes: usize,
    ) -> Result<StagedDynamicSceneSpawn, DynamicSceneError> {
        let target = self.capture_level_target(level, target_snapshot_limit_bytes)?;
        self.stage_target(target)
    }

    pub(crate) fn capture_level_target(
        &self,
        level: &LevelSystem,
        target_snapshot_limit_bytes: usize,
    ) -> Result<DynamicSceneSpawnTargetSnapshot, DynamicSceneError> {
        let (target_level, expected_generation, preflight_world, plan, estimated_bytes) =
            level.dynamic_scene_preflight_snapshot(&self.scene, target_snapshot_limit_bytes)?;
        Ok(DynamicSceneSpawnTargetSnapshot {
            expected_generation,
            target_level: Some(target_level),
            preflight_world,
            plan,
            estimated_bytes,
        })
    }

    pub(crate) fn stage_target(
        self,
        target: DynamicSceneSpawnTargetSnapshot,
    ) -> Result<StagedDynamicSceneSpawn, DynamicSceneError> {
        let component_type_count = self.component_type_count;
        let entity_count = self.entity_count;
        let resource_count = self.resource_count;
        let DynamicSceneSpawnTargetSnapshot {
            expected_generation,
            target_level,
            mut preflight_world,
            plan,
            estimated_bytes: _,
        } = target;
        let mutation = DynamicScene::validate_compiled_spawn_preflight(&mut preflight_world, plan)?;
        Ok(StagedDynamicSceneSpawn {
            expected_generation,
            target_level,
            mutation,
            component_type_count,
            entity_count,
            resource_count,
        })
    }
}

pub(crate) struct DynamicSceneSpawnTargetSnapshot {
    expected_generation: u64,
    target_level: Option<WorldHandle>,
    preflight_world: World,
    plan: CompiledSceneSpawn,
    estimated_bytes: usize,
}

impl DynamicSceneSpawnTargetSnapshot {
    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }
}

pub(crate) struct StagedDynamicSceneSpawn {
    expected_generation: u64,
    target_level: Option<WorldHandle>,
    mutation: PreflightedSceneMutation,
    component_type_count: usize,
    entity_count: usize,
    resource_count: usize,
}

impl StagedDynamicSceneSpawn {
    pub(crate) fn component_type_count(&self) -> usize {
        self.component_type_count
    }

    pub(crate) fn entity_count(&self) -> usize {
        self.entity_count
    }

    pub(crate) fn resource_count(&self) -> usize {
        self.resource_count
    }

    pub(crate) fn commit_into(self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        let Self {
            expected_generation: _,
            target_level: _,
            mutation,
            component_type_count: _,
            entity_count: _,
            resource_count: _,
        } = self;
        DynamicScene::commit_preflighted_spawn_into(world, mutation)
    }

    pub(crate) fn commit_into_level(
        self,
        level: &LevelSystem,
    ) -> Result<EntityRemap, DynamicSceneError> {
        let Self {
            expected_generation,
            target_level,
            mutation,
            component_type_count: _,
            entity_count: _,
            resource_count: _,
        } = self;
        let actual_level = level.world_handle();
        if target_level != Some(actual_level) {
            return Err(DynamicSceneError::TargetLevelChanged {
                expected: format!("{:?}", target_level),
                actual: format!("{:?}", actual_level),
            });
        }
        level.commit_preflighted_dynamic_scene_if_generation(expected_generation, mutation)
    }
}

fn estimate_scene_bytes(scene: &DynamicScene) -> Result<usize, DynamicSceneError> {
    let mut counter = ByteCounter::default();
    serde_json::to_writer(&mut counter, scene).map_err(|error| {
        DynamicSceneError::PreparedSizeEstimation {
            reason: error.to_string(),
        }
    })?;

    Ok(counter
        .bytes
        .saturating_mul(2)
        .saturating_add(std::mem::size_of::<DynamicScene>()))
}

#[derive(Default)]
struct ByteCounter {
    bytes: usize,
}

#[cfg(test)]
#[path = "tests/prepared.rs"]
mod tests;

impl Write for ByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes = self.bytes.saturating_add(buffer.len());
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
