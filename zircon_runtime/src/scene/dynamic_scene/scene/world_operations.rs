use crate::scene::World;

use super::super::{DynamicSceneError, EntityRemap, ScenePatchPreviewReport};
use super::{DynamicScene, capture, spawn, validation};

impl DynamicScene {
    pub fn from_world(world: &World) -> Result<Self, DynamicSceneError> {
        capture::dynamic_scene_from_world(world)
    }

    pub fn spawn_into(&self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        spawn::spawn_scene_into(self, world)
    }

    pub fn preview_spawn_into(
        &self,
        world: &World,
    ) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
        spawn::preview_scene_spawn_into(self, world)
    }

    pub(crate) fn compile_spawn_into(
        &self,
        world: &World,
    ) -> Result<spawn::CompiledSceneSpawn, DynamicSceneError> {
        spawn::compile_scene_spawn(self, world)
    }

    pub(crate) fn apply_compiled_spawn_into(
        world: &mut World,
        plan: spawn::CompiledSceneSpawn,
    ) -> Result<EntityRemap, DynamicSceneError> {
        spawn::apply_compiled_scene_spawn(world, plan)
    }

    pub(crate) fn capture_compiled_spawn_preflight(
        world: &World,
        plan: &spawn::CompiledSceneSpawn,
        limit_bytes: usize,
    ) -> Result<(World, usize), DynamicSceneError> {
        spawn::capture_compiled_scene_spawn_preflight(world, plan, limit_bytes)
    }

    pub(crate) fn validate_compiled_spawn_preflight(
        preflight: &mut World,
        plan: spawn::CompiledSceneSpawn,
    ) -> Result<spawn::PreflightedSceneMutation, DynamicSceneError> {
        spawn::validate_compiled_scene_spawn_preflight(preflight, plan)
    }

    pub(crate) fn commit_preflighted_spawn_into(
        world: &mut World,
        mutation: spawn::PreflightedSceneMutation,
    ) -> Result<EntityRemap, DynamicSceneError> {
        spawn::commit_preflighted_compiled_scene_spawn(world, mutation)
    }

    pub(crate) fn stage_existing_resources_bounded(
        &self,
        source: &World,
        target: &mut World,
        base_estimated_bytes: usize,
        limit_bytes: usize,
    ) -> Result<usize, DynamicSceneError> {
        spawn::stage_existing_resources_bounded(
            self,
            source,
            target,
            base_estimated_bytes,
            limit_bytes,
        )
    }

    pub fn ensure_supported(&self) -> Result<(), DynamicSceneError> {
        validation::ensure_scene_supported(self)
    }
}
