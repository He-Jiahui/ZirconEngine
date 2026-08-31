use std::sync::Arc;

use zircon_runtime::builtin::RuntimeModuleCompositionIdentity;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::plugin::native::{
    host::NativePluginHostHandle, NativePluginBehaviorCallReport,
    NativePluginRuntimeBehaviorDescriptor, NativePluginRuntimeCommandDispatchReport,
    NativePluginRuntimePlayModeExitReport, NativePluginRuntimePlayModeSnapshot,
    NativePluginRuntimeStateRestoreReport, NativePluginRuntimeStateSnapshot,
};
use zircon_runtime::plugin::{CompiledProjectPluginPlan, RuntimePluginBridgeLifecycleState};

use super::super::{EntryModuleSelectionReport, ResolvedProductHostConfig};

/// One admitted and bootstrapped product generation with every owner needed to keep it valid.
#[must_use = "the product composition owns Core and plugin lifetimes for this generation"]
#[derive(Debug)]
pub struct ProductComposition {
    resolved_config: ResolvedProductHostConfig,
    module_selection_report: EntryModuleSelectionReport,
    diagnostics: Vec<String>,
    // Core must release its runtime graph before native dynamic-library handles are dropped.
    core: CoreHandle,
    plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
    compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
    native_plugin_host: Option<NativePluginHostHandle>,
}

impl ProductComposition {
    pub(super) fn new(
        resolved_config: ResolvedProductHostConfig,
        module_selection_report: EntryModuleSelectionReport,
        diagnostics: Vec<String>,
        core: CoreHandle,
        plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
        compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
        native_plugin_host: Option<NativePluginHostHandle>,
    ) -> Self {
        Self {
            resolved_config,
            module_selection_report,
            diagnostics,
            core,
            plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan,
            native_plugin_host,
        }
    }

    /// Returns the admitted product configuration used for this generation.
    pub const fn resolved_config(&self) -> &ResolvedProductHostConfig {
        &self.resolved_config
    }

    /// Borrows the bootstrapped Core inside the App-owned host boundary.
    pub(crate) const fn core(&self) -> &CoreHandle {
        &self.core
    }

    /// Returns the module selection receipt captured before Core bootstrap.
    pub const fn module_selection_report(&self) -> &EntryModuleSelectionReport {
        &self.module_selection_report
    }

    /// Returns the stable identity of the runtime module composition.
    pub const fn runtime_module_composition_identity(&self) -> &RuntimeModuleCompositionIdentity {
        &self
            .module_selection_report
            .runtime_module_composition_identity
    }

    /// Returns the compiled project plugin plan retained by this generation.
    pub fn compiled_project_plugin_plan(&self) -> Option<&CompiledProjectPluginPlan> {
        self.compiled_project_plugin_plan.as_deref()
    }

    /// Returns the retained runtime plugin bridge lifecycle state, when present.
    pub const fn runtime_plugin_bridge_lifecycle_state(
        &self,
    ) -> Option<&RuntimePluginBridgeLifecycleState> {
        self.plugin_bridge_lifecycle_state.as_ref()
    }

    /// Returns the live native plugin host owner, when native discovery was requested.
    pub const fn native_plugin_host(&self) -> Option<&NativePluginHostHandle> {
        self.native_plugin_host.as_ref()
    }

    /// Returns non-fatal diagnostics collected while preparing this generation.
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    /// Queries one loaded native runtime behavior descriptor.
    pub fn runtime_behavior_descriptor(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeBehaviorDescriptor, String> {
        self.require_native_plugin_host()?
            .runtime_behavior_descriptor(plugin_id)
    }

    /// Lists every loaded native runtime behavior descriptor.
    pub fn runtime_behavior_descriptors(
        &self,
    ) -> Result<Vec<NativePluginRuntimeBehaviorDescriptor>, String> {
        self.require_native_plugin_host()?
            .runtime_behavior_descriptors()
    }

    /// Invokes a command on one loaded native runtime plugin.
    pub fn invoke_runtime_plugin_command(
        &self,
        plugin_id: impl AsRef<str>,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.require_native_plugin_host()?
            .invoke_runtime_plugin_command(plugin_id, command_name, payload)
    }

    /// Dispatches a command to all interested loaded native runtime plugins.
    pub fn dispatch_runtime_plugin_command(
        &self,
        command_name: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
    ) -> Result<NativePluginRuntimeCommandDispatchReport, String> {
        self.require_native_plugin_host()?
            .dispatch_runtime_plugin_command(command_name, payload)
    }

    /// Saves state for one loaded native runtime plugin.
    pub fn save_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.require_native_plugin_host()?
            .save_runtime_plugin_state(plugin_id)
    }

    /// Saves state for all loaded native runtime plugins.
    pub fn save_runtime_plugin_states(&self) -> Result<NativePluginRuntimeStateSnapshot, String> {
        self.require_native_plugin_host()?
            .save_runtime_plugin_states()
    }

    /// Restores state for one loaded native runtime plugin.
    pub fn restore_runtime_plugin_state(
        &self,
        plugin_id: impl AsRef<str>,
        state: impl AsRef<[u8]>,
    ) -> Result<NativePluginBehaviorCallReport, String> {
        self.require_native_plugin_host()?
            .restore_runtime_plugin_state(plugin_id, state)
    }

    /// Restores a previously captured native runtime plugin state snapshot.
    pub fn restore_runtime_plugin_states(
        &self,
        snapshot: &NativePluginRuntimeStateSnapshot,
    ) -> Result<NativePluginRuntimeStateRestoreReport, String> {
        self.require_native_plugin_host()?
            .restore_runtime_plugin_states(snapshot)
    }

    /// Captures native runtime plugin state and enters play mode.
    pub fn enter_runtime_play_mode(&self) -> Result<NativePluginRuntimePlayModeSnapshot, String> {
        self.require_native_plugin_host()?.enter_runtime_play_mode()
    }

    /// Exits play mode and restores the supplied native runtime plugin state.
    pub fn exit_runtime_play_mode(
        &self,
        snapshot: &NativePluginRuntimePlayModeSnapshot,
    ) -> Result<NativePluginRuntimePlayModeExitReport, String> {
        self.require_native_plugin_host()?
            .exit_runtime_play_mode(snapshot)
    }

    fn require_native_plugin_host(&self) -> Result<&NativePluginHostHandle, String> {
        self.native_plugin_host.as_ref().ok_or_else(|| {
            "product composition does not own a native plugin host for this generation".to_owned()
        })
    }
}
