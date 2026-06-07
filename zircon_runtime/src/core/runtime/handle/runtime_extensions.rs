use std::sync::Arc;

use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, SceneRuntimeHookRegistration,
};
use crate::scene::SystemStage;

use super::super::state::SceneRuntimeHookStagePlan;
use super::CoreHandle;

impl CoreHandle {
    pub fn install_scene_runtime_hooks(
        &self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut registry = RuntimeExtensionRegistry::default();
        {
            let hooks = self.inner.scene_hooks.lock().unwrap();
            for hook in hooks.ordered().iter().cloned() {
                registry.register_scene_hook(hook)?;
            }
        }
        for hook in extensions.scene_hooks().iter().cloned() {
            registry.register_scene_hook(hook)?;
        }
        *self.inner.scene_hooks.lock().unwrap() =
            super::super::state::SceneRuntimeHookSet::from_ordered(registry.scene_hooks().to_vec());
        Ok(())
    }

    pub fn scene_runtime_hooks_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<SceneRuntimeHookRegistration> {
        self.inner
            .scene_hooks
            .lock()
            .unwrap()
            .hooks_for_stage(stage)
            .to_vec()
    }

    pub(crate) fn scene_runtime_hook_stage_plan_snapshot(&self) -> Arc<SceneRuntimeHookStagePlan> {
        self.inner.scene_hooks.lock().unwrap().stage_plan()
    }
}
