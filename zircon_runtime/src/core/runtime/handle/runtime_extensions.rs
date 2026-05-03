use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, SceneRuntimeHookRegistration,
};
use crate::scene::components::SystemStage;

use super::CoreHandle;

impl CoreHandle {
    pub fn install_scene_runtime_hooks(
        &self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut registry = RuntimeExtensionRegistry::default();
        {
            let hooks = self.inner.scene_hooks.lock().unwrap();
            for hook in hooks.iter().cloned() {
                registry.register_scene_hook(hook)?;
            }
        }
        for hook in extensions.scene_hooks().iter().cloned() {
            registry.register_scene_hook(hook)?;
        }
        *self.inner.scene_hooks.lock().unwrap() = registry.scene_hooks().to_vec();
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
            .iter()
            .filter(|hook| hook.descriptor().stage == stage)
            .cloned()
            .collect()
    }
}
