use std::sync::Arc;

use crate::core::CoreError;
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleOutcome, RuntimePluginBridgeLifecycleState,
    SceneRuntimeHookRegistration,
};
use crate::scene::SystemStage;

use super::super::state::SceneRuntimeHookStagePlan;
use super::CoreHandle;

impl CoreHandle {
    pub fn install_world_runtime_extensions(
        &self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.lock_world_extensions().install(extensions)
    }

    pub(crate) fn apply_world_runtime_extensions(
        &self,
        world: &mut crate::scene::World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.lock_world_extensions().apply_to_world(world)
    }

    pub fn install_plugin_bridge_lifecycle_state(&self, state: RuntimePluginBridgeLifecycleState) {
        *self.lock_plugin_bridge_lifecycle() = Some(state);
    }

    pub fn clear_plugin_bridge_lifecycle_state(&self) -> Option<RuntimePluginBridgeLifecycleState> {
        self.lock_plugin_bridge_lifecycle().take()
    }

    pub fn plugin_bridge_lifecycle_state(&self) -> Option<RuntimePluginBridgeLifecycleState> {
        self.lock_plugin_bridge_lifecycle().clone()
    }

    pub fn apply_plugin_bridge_lifecycle_event(
        &self,
        event: RuntimePluginBridgeLifecycleEvent,
    ) -> Option<RuntimePluginBridgeLifecycleOutcome> {
        self.plugin_bridge_lifecycle_state()
            .map(|state| state.apply_provider_lifecycle_event(event))
    }

    pub fn activate_plugin_bridge_provider_at_frame_boundary(
        &self,
        provider_package_id: impl Into<String>,
    ) -> Option<RuntimePluginBridgeLifecycleOutcome> {
        self.apply_plugin_bridge_lifecycle_event(
            RuntimePluginBridgeLifecycleEvent::activate_provider(provider_package_id),
        )
    }

    pub fn disable_plugin_bridge_provider_at_frame_boundary(
        &self,
        provider_package_id: impl Into<String>,
    ) -> Option<RuntimePluginBridgeLifecycleOutcome> {
        self.apply_plugin_bridge_lifecycle_event(
            RuntimePluginBridgeLifecycleEvent::disable_provider(provider_package_id),
        )
    }

    pub fn deactivate_plugin_bridge_provider_at_frame_boundary(
        &self,
        provider_package_id: impl Into<String>,
    ) -> Option<RuntimePluginBridgeLifecycleOutcome> {
        self.apply_plugin_bridge_lifecycle_event(
            RuntimePluginBridgeLifecycleEvent::deactivate_provider(provider_package_id),
        )
    }

    pub fn plugin_bridge_provider_package_id_for_runtime_module(
        &self,
        runtime_module_name: &str,
    ) -> Option<String> {
        self.plugin_bridge_lifecycle_state()
            .and_then(|state| state.provider_package_id_for_runtime_module(runtime_module_name))
    }

    pub(crate) fn activate_plugin_bridge_provider_for_runtime_module(
        &self,
        runtime_module_name: &str,
    ) -> Option<RuntimePluginBridgeLifecycleOutcome> {
        let provider_package_id =
            self.plugin_bridge_provider_package_id_for_runtime_module(runtime_module_name)?;
        self.activate_plugin_bridge_provider_at_frame_boundary(provider_package_id)
    }

    pub(crate) fn deactivate_plugin_bridge_provider_for_runtime_module(
        &self,
        runtime_module_name: &str,
    ) -> Result<Option<RuntimePluginBridgeLifecycleOutcome>, CoreError> {
        let Some(provider_package_id) =
            self.plugin_bridge_provider_package_id_for_runtime_module(runtime_module_name)
        else {
            return Ok(None);
        };
        match self.deactivate_plugin_bridge_provider_at_frame_boundary(provider_package_id) {
            Some(RuntimePluginBridgeLifecycleOutcome::Blocked(error)) => {
                Err(CoreError::PluginBridgeLifecycleBlocked(error.diagnostic()))
            }
            outcome => Ok(outcome),
        }
    }

    pub fn install_scene_runtime_hooks(
        &self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut registry = RuntimeExtensionRegistry::default();
        {
            let hooks = self.lock_scene_hooks();
            for hook in hooks.ordered().iter().cloned() {
                registry.register_scene_hook(hook)?;
            }
        }
        for hook in extensions.scene_hooks().iter().cloned() {
            registry.register_scene_hook(hook)?;
        }
        *self.lock_scene_hooks() =
            super::super::state::SceneRuntimeHookSet::from_ordered(registry.scene_hooks().to_vec());
        Ok(())
    }

    pub fn scene_runtime_hooks_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<SceneRuntimeHookRegistration> {
        self.lock_scene_hooks().hooks_for_stage(stage).to_vec()
    }

    pub(crate) fn scene_runtime_hook_stage_plan_snapshot(&self) -> Arc<SceneRuntimeHookStagePlan> {
        self.lock_scene_hooks().stage_plan()
    }
}
