use std::io::{self, Write};

use crate::{
    core::framework::scene::WorldHandle,
    scene::{EntityRemap, LevelSystem, World},
};

use super::super::{scene::CompiledSceneSpawn, DynamicScene, DynamicSceneError};

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
        let (preflight_world, estimated_bytes) = self.scene.capture_compiled_spawn_preflight(
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
        self.scene
            .validate_compiled_spawn_preflight(&mut preflight_world, &plan)?;
        Ok(StagedDynamicSceneSpawn {
            expected_generation,
            target_level,
            scene: self.scene,
            plan,
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
    scene: DynamicScene,
    plan: CompiledSceneSpawn,
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
            scene,
            plan,
            component_type_count: _,
            entity_count: _,
            resource_count: _,
        } = self;
        scene.apply_preflighted_compiled_spawn_into(world, plan)
    }

    pub(crate) fn commit_into_level(
        self,
        level: &LevelSystem,
    ) -> Result<EntityRemap, DynamicSceneError> {
        let Self {
            expected_generation,
            target_level,
            scene,
            plan,
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
        level.apply_preflighted_dynamic_scene_if_generation(expected_generation, &scene, plan)
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
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::scene::{
        components::Name, ecs::LifecycleEventKind, DefaultLevelManager, DynamicScene,
        DynamicSceneError, NodeKind, Resource, World,
    };

    use super::PreparedDynamicSceneSpawn;

    #[derive(Debug, PartialEq, Eq)]
    struct UnrelatedRuntimeResource(u32);

    impl Resource for UnrelatedRuntimeResource {}

    #[test]
    fn dynamic_scene_asset_reload_staged_spawn_rejects_changed_target() {
        let mut source = World::empty();
        source.spawn_node(NodeKind::Empty);
        let scene = DynamicScene::from_world(&source).expect("source scene should capture");
        let prepared =
            PreparedDynamicSceneSpawn::new(scene.clone()).expect("captured scene should prepare");
        let mut target = World::empty();
        let expected_generation = target.world_generation();
        let staged = prepared
            .stage_into(&mut target)
            .expect("scene should stage on target snapshot");

        target.spawn_node(NodeKind::Cube);
        let changed_target = target.clone();
        let actual_generation = target.world_generation();
        let error = staged
            .commit_into(&mut target)
            .expect_err("stale transaction must not replace a changed target");

        assert_eq!(
            error,
            DynamicSceneError::TargetWorldChanged {
                expected_generation,
                actual_generation,
            }
        );
        assert_eq!(target, changed_target);
    }

    #[test]
    fn dynamic_scene_asset_reload_staged_spawn_preserves_live_runtime_resources() {
        let mut source = World::empty();
        source.spawn_node(NodeKind::Empty);
        let prepared = PreparedDynamicSceneSpawn::new(
            DynamicScene::from_world(&source).expect("source scene should capture"),
        )
        .expect("captured scene should prepare");
        let mut target = World::empty();
        target.insert_resource(UnrelatedRuntimeResource(41));

        prepared
            .spawn_into(&mut target)
            .expect("staged scene should commit");

        assert_eq!(
            target.get_resource::<UnrelatedRuntimeResource>(),
            Some(&UnrelatedRuntimeResource(41))
        );
    }

    #[test]
    fn dynamic_scene_asset_reload_level_transaction_is_target_bound_and_replays_live_callbacks() {
        let mut source = World::empty();
        source.spawn_node(NodeKind::Empty);
        let scene = DynamicScene::from_world(&source).expect("source scene should capture");
        let prepared =
            PreparedDynamicSceneSpawn::new(scene.clone()).expect("captured scene should prepare");
        let manager = DefaultLevelManager::default();
        let first = manager.create_level(World::empty(), Default::default());
        let second = manager.create_level(World::empty(), Default::default());
        let callbacks = Arc::new(AtomicUsize::new(0));
        let observed = callbacks.clone();
        first.with_world_mut(|world| {
            world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |_, _| {
                observed.fetch_add(1, Ordering::Relaxed);
            });
        });

        let staged = prepared
            .stage_into_level(&first, 1024 * 1024)
            .expect("level scene should stage outside commit");
        let error = staged
            .commit_into_level(&second)
            .expect_err("one level transaction must not commit into another level");

        assert!(matches!(
            error,
            DynamicSceneError::TargetLevelChanged { .. }
        ));
        assert_eq!(callbacks.load(Ordering::Relaxed), 0);
        assert!(second.with_world(|world| world.node_records().is_empty()));

        PreparedDynamicSceneSpawn::new(scene)
            .expect("captured scene should prepare again")
            .stage_into_level(&first, 1024 * 1024)
            .expect("first level should stage")
            .commit_into_level(&first)
            .expect("target-bound transaction should commit to its source level");
        assert!(callbacks.load(Ordering::Relaxed) > 0);
    }
}

impl Write for ByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes = self.bytes.saturating_add(buffer.len());
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
