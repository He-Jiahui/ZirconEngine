use crate::plugin::{RuntimeExtensionRegistryError, SceneRuntimeHookRegistration};
use crate::scene::SystemStage;

use super::super::validation::validate_scene_hook_registration;
use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn register_scene_hook(
        &mut self,
        registration: SceneRuntimeHookRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_scene_hook_registration(&registration)?;
        let id = registration.descriptor().id.as_str();
        if self
            .scene_hooks
            .iter()
            .any(|existing| existing.descriptor().id == id)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateSceneHook(
                id.to_string(),
            ));
        }
        self.scene_hooks.push(registration);
        sort_scene_hooks(&mut self.scene_hooks);
        Ok(())
    }
}

fn sort_scene_hooks(hooks: &mut [SceneRuntimeHookRegistration]) {
    hooks.sort_by(|left, right| {
        scene_stage_rank(left.descriptor().stage)
            .cmp(&scene_stage_rank(right.descriptor().stage))
            .then(left.descriptor().order.cmp(&right.descriptor().order))
            .then(
                left.descriptor()
                    .id
                    .as_str()
                    .cmp(right.descriptor().id.as_str()),
            )
    });
}

fn scene_stage_rank(stage: SystemStage) -> usize {
    stage.rank()
}
