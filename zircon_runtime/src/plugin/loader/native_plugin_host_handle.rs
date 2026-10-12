use std::path::Path;
use std::sync::{Arc, Weak};

use crate::plugin::{
    PluginModuleKind, RuntimeExtensionRegistry, RuntimePluginBridgeLifecycleState,
};

use super::{
    NativeBridgeMethodBinding, NativeHostBridgeCallScope, NativePluginArtifactAuthority,
    NativePluginBehaviorCallReport, NativePluginCallbackDiagnostics, NativePluginLiveHost,
    NativePluginLiveHostBridgeReloadReport, NativePluginLiveHostDiagnostics,
    NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginProjectActivationCleanupReceipt, NativePluginProjectActivationRequest,
    NativePluginProjectActivationResult, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimeDeltaHotUpdateReport,
    NativePluginRuntimeDeltaHotUpdateRequest, NativePluginRuntimeHotUpdateReport,
    NativePluginRuntimePlayModeExitReport, NativePluginRuntimePlayModeSnapshot,
    NativePluginRuntimeRegistrationReplayReport, NativePluginRuntimeStateRestoreReport,
    NativePluginRuntimeStateSnapshot,
};

#[derive(Clone, Debug, Default)]
pub struct NativePluginHostHandle {
    backend: Arc<NativePluginLiveHost>,
}

impl NativePluginHostHandle {
    pub fn with_artifact_authority(artifact_authority: NativePluginArtifactAuthority) -> Self {
        Self {
            backend: Arc::new(NativePluginLiveHost::with_artifact_authority(
                artifact_authority,
            )),
        }
    }

    pub fn artifact_authority(&self) -> &NativePluginArtifactAuthority {
        self.backend.artifact_authority()
    }

    pub fn downgrade(&self) -> NativePluginHostWeakHandle {
        NativePluginHostWeakHandle {
            backend: Arc::downgrade(&self.backend),
        }
    }

    pub fn live_host_diagnostics(&self) -> NativePluginLiveHostDiagnostics {
        self.backend.live_host_diagnostics()
    }

    pub fn plugin_callback_diagnostics(
        &self,
        plugin_id: impl AsRef<str>,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginCallbackDiagnostics, String> {
        self.backend
            .plugin_callback_diagnostics(plugin_id, module_kind)
    }

    pub fn load_runtime_plugins_from_project_root(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend.load_runtime_plugins_from_project_root(root)
    }

    pub fn load_runtime_plugins_from_project_root_with_authority(
        &self,
        root: impl AsRef<Path>,
        authority: &NativePluginArtifactAuthority,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend
            .load_runtime_plugins_from_project_root_with_authority(root, authority)
    }

    pub fn activate_runtime_project_plugins(
        &self,
        request: NativePluginProjectActivationRequest<'_>,
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginProjectActivationResult {
        self.backend
            .activate_runtime_project_plugins(request, authority)
    }

    pub fn retry_project_activation_recovery(&self) -> NativePluginProjectActivationCleanupReceipt {
        self.backend.retry_project_activation_recovery()
    }

    pub fn load_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend
            .load_runtime_plugins_from_export_root(export_root)
    }

    pub fn load_editor_plugins_from_project_root(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend.load_editor_plugins_from_project_root(root)
    }

    pub fn load_editor_plugins_from_project_root_with_authority(
        &self,
        root: impl AsRef<Path>,
        authority: &NativePluginArtifactAuthority,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend
            .load_editor_plugins_from_project_root_with_authority(root, authority)
    }

    pub fn load_editor_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend
            .load_editor_plugins_from_export_root(export_root)
    }

    pub fn hot_reload_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginRuntimeHotUpdateReport, String> {
        self.backend
            .hot_reload_runtime_plugins_from_export_root(export_root)
    }

    pub fn hot_reload_runtime_plugins_after_delta_pack_install(
        &self,
        request: NativePluginRuntimeDeltaHotUpdateRequest,
    ) -> Result<NativePluginRuntimeDeltaHotUpdateReport, String> {
        self.backend
            .hot_reload_runtime_plugins_after_delta_pack_install(request)
    }

    pub fn loaded_plugin_ids(&self, module_kind: PluginModuleKind) -> Result<Vec<String>, String> {
        self.backend.loaded_plugin_ids(module_kind)
    }

    pub fn load_runtime_plugins_from_project_root_with_bridge_lifecycle(
        &self,
        root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend
            .load_runtime_plugins_from_project_root_with_bridge_lifecycle(root, lifecycle)
    }

    pub fn load_runtime_plugins_from_export_root_with_bridge_lifecycle(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.backend
            .load_runtime_plugins_from_export_root_with_bridge_lifecycle(export_root, lifecycle)
    }

    pub fn hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginRuntimeHotUpdateReport, String> {
        self.backend
            .hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle(
                export_root,
                lifecycle,
            )
    }

    pub fn unload_runtime_plugin_with_bridge_lifecycle(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend
            .unload_runtime_plugin_with_bridge_lifecycle(plugin_id, lifecycle)
    }

    pub fn hot_reload_runtime_plugin_with_bridge_lifecycle(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend
            .hot_reload_runtime_plugin_with_bridge_lifecycle(root, plugin_id, lifecycle)
    }

    pub fn install_discovered_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<usize, String> {
        self.backend
            .install_discovered_runtime_bridge_method_bindings(plugin_id)
    }

    pub fn install_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> Result<(), String> {
        self.backend
            .install_runtime_bridge_method_bindings(plugin_id, bindings)
    }

    pub fn clear_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<bool, String> {
        self.backend.clear_runtime_bridge_method_bindings(plugin_id)
    }

    pub fn runtime_bridge_call_scope_from_installed_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativeHostBridgeCallScope, String> {
        self.backend
            .runtime_bridge_call_scope_from_installed_bindings(plugin_id, lifecycle)
    }

    pub fn reload_runtime_bridge_provider_and_scope_from_installed_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostBridgeReloadReport, String> {
        self.backend
            .reload_runtime_bridge_provider_and_scope_from_installed_bindings(plugin_id, lifecycle)
    }

    pub fn replay_runtime_registration_manifests_via_bridge(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginRuntimeRegistrationReplayReport, String> {
        self.backend
            .replay_runtime_registration_manifests_via_bridge(registry, lifecycle)
    }

    pub fn replay_runtime_plugin_registration_manifest_via_bridge(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeRegistrationReplayReport, String> {
        self.backend
            .replay_runtime_plugin_registration_manifest_via_bridge(registry, lifecycle, plugin_id)
    }

    pub fn enter_runtime_play_mode(&self) -> Result<NativePluginRuntimePlayModeSnapshot, String> {
        self.backend.enter_runtime_play_mode()
    }

    pub fn runtime_behavior_descriptor(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeBehaviorDescriptor, String> {
        self.backend.runtime_behavior_descriptor(plugin_id)
    }

    pub fn runtime_behavior_descriptors(
        &self,
    ) -> Result<Vec<NativePluginRuntimeBehaviorDescriptor>, String> {
        self.backend.runtime_behavior_descriptors()
    }

    pub fn invoke_runtime_plugin_command(
        &self,
        plugin_id: impl AsRef<str>,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.backend
            .invoke_runtime_plugin_command(plugin_id, command_name, payload)
    }

    pub fn dispatch_runtime_plugin_command(
        &self,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginRuntimeCommandDispatchReport, String> {
        self.backend
            .dispatch_runtime_plugin_command(command_name, payload)
    }

    pub fn save_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.backend.save_runtime_plugin_state(plugin_id)
    }

    pub fn save_runtime_plugin_states(&self) -> Result<NativePluginRuntimeStateSnapshot, String> {
        self.backend.save_runtime_plugin_states()
    }

    pub fn restore_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
        state: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.backend.restore_runtime_plugin_state(plugin_id, state)
    }

    pub fn restore_runtime_plugin_states(
        &self,
        snapshot: &NativePluginRuntimeStateSnapshot,
    ) -> Result<NativePluginRuntimeStateRestoreReport, String> {
        self.backend.restore_runtime_plugin_states(snapshot)
    }

    pub fn exit_runtime_play_mode(
        &self,
        snapshot: &NativePluginRuntimePlayModeSnapshot,
    ) -> Result<NativePluginRuntimePlayModeExitReport, String> {
        self.backend.exit_runtime_play_mode(snapshot)
    }

    pub fn hot_reload_editor_plugin(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend.hot_reload_editor_plugin(root, plugin_id)
    }

    pub fn hot_reload_editor_plugin_with_authority(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
        authority: &NativePluginArtifactAuthority,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend
            .hot_reload_editor_plugin_with_authority(root, plugin_id, authority)
    }

    pub fn hot_reload_runtime_plugin(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend.hot_reload_runtime_plugin(root, plugin_id)
    }

    pub fn hot_reload_runtime_plugin_with_authority(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
        authority: &NativePluginArtifactAuthority,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend
            .hot_reload_runtime_plugin_with_authority(root, plugin_id, authority)
    }

    pub fn unload_runtime_plugin(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend.unload_runtime_plugin(plugin_id)
    }

    pub fn unload_editor_plugin(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.backend.unload_editor_plugin(plugin_id)
    }
}

#[derive(Clone, Debug, Default)]
pub struct NativePluginHostWeakHandle {
    backend: Weak<NativePluginLiveHost>,
}

impl NativePluginHostWeakHandle {
    pub fn upgrade(&self) -> Option<NativePluginHostHandle> {
        self.backend
            .upgrade()
            .map(|backend| NativePluginHostHandle { backend })
    }
}

#[cfg(test)]
#[path = "tests/native_plugin_host_handle.rs"]
mod tests;
