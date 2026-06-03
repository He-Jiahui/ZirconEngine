use crate::plugin::SceneRuntimeHookRegistration;

use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn scene_hooks(&self) -> &[SceneRuntimeHookRegistration] {
        &self.scene_hooks
    }
}
