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
        let id = registration.descriptor().id.clone();
        if self.scene_hooks.contains_key(&id) {
            return Err(RuntimeExtensionRegistryError::DuplicateSceneHook(id));
        }
        let owner = self.intern_runtime_owner(&registration.descriptor().plugin_id)?;
        self.scene_hooks
            .register(owner, id, registration)
            .expect("scene hook duplicate was prechecked");
        self.scene_hooks.sort_by_values(compare_scene_hooks);
        Ok(())
    }
}

fn compare_scene_hooks(
    left: &SceneRuntimeHookRegistration,
    right: &SceneRuntimeHookRegistration,
) -> std::cmp::Ordering {
    scene_stage_rank(left.descriptor().stage)
        .cmp(&scene_stage_rank(right.descriptor().stage))
        .then(left.descriptor().order.cmp(&right.descriptor().order))
        .then(
            left.descriptor()
                .id
                .as_str()
                .cmp(right.descriptor().id.as_str()),
        )
}

fn scene_stage_rank(stage: SystemStage) -> usize {
    stage.rank()
}
