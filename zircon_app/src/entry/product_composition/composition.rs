use std::sync::Arc;

use std::error::Error;
use std::time::Instant;
use zircon_runtime::builtin::RuntimeModuleCompositionIdentity;
use zircon_runtime::core::{CoreHandle, CoreRuntime, TaskGraphShutdownReport};

use super::ownership::ProductOwnership;
use crate::entry::product_shutdown::retained_owner::{ProductCompositionFailure, RetainedPacket};
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
    // Moving this exact packet preserves Core and every generation pin together.
    ownership: Option<ProductOwnership>,
}

impl ProductComposition {
    fn ownership(&self) -> &ProductOwnership {
        self.ownership
            .as_ref()
            .expect("a live composition retains its original packet")
    }

    pub(super) fn runtime(&self) -> &CoreRuntime {
        &self.ownership().runtime
    }

    pub(in crate::entry) fn into_packet(mut self) -> RetainedPacket {
        RetainedPacket::product(
            self.ownership
                .take()
                .expect("composition ownership transfers once"),
        )
    }

    /// Closes project watchers, original modules and the Core-owned graph under one cooperative
    /// deadline. Failure retains the exact original packet for explicit retry. External handles
    /// and arbitrary native workers are not certified by the returned graph report.
    pub fn close_until(
        self,
        deadline: Instant,
    ) -> Result<TaskGraphShutdownReport, ProductCompositionFailure> {
        let mut packet = self.into_packet();
        match packet.close_until(deadline) {
            Ok(report) => Ok(report.expect("a product packet has an actual Core graph report")),
            Err(error) => Err(ProductCompositionFailure::retained(Arc::new(error), packet)),
        }
    }

    /// Preserves an ordinary startup/host primary while closing the same acquired generation.
    /// Cleanup success leaves the original Result failed; incomplete cleanup remains retained.
    pub fn fail_until(
        self,
        primary: impl Error + Send + Sync + 'static,
        deadline: Instant,
    ) -> ProductCompositionFailure {
        ProductCompositionFailure::owned(Arc::new(primary), self.into_packet(), deadline)
    }

    pub(in crate::entry) fn fail_with_runtime_until(
        mut self,
        failure: crate::entry::runtime_library::RuntimeSessionCreateFailure,
        deadline: Instant,
    ) -> ProductCompositionFailure {
        let product = self.ownership.take().expect("composition transfers once");
        if let Some(owner) = failure.retained_owner() {
            match owner.attach_product(product) {
                Ok(()) => {
                    let result = ProductCompositionFailure::from_owner(owner);
                    let _ = result.retry_cleanup_until(deadline);
                    return result;
                }
                Err(product) => {
                    return ProductCompositionFailure::owned(
                        Arc::new(failure),
                        RetainedPacket::product(product),
                        deadline,
                    );
                }
            }
        }
        ProductCompositionFailure::owned(
            Arc::new(failure),
            RetainedPacket::product(product),
            deadline,
        )
    }

    pub(crate) fn retain_plugin_selection_outcomes(
        &mut self,
        outcomes: impl IntoIterator<
            Item = zircon_runtime::core::framework::project::PluginSelectionResolution,
        >,
    ) {
        for outcome in outcomes {
            let already_retained = self
                .module_selection_report
                .plugin_selection_outcomes
                .iter()
                .any(|retained| retained == &outcome);
            if !already_retained {
                self.module_selection_report
                    .plugin_selection_outcomes
                    .push(outcome);
            }
        }
    }

    pub(super) fn new(
        resolved_config: ResolvedProductHostConfig,
        module_selection_report: EntryModuleSelectionReport,
        diagnostics: Vec<String>,
        runtime: CoreRuntime,
        plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
        compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
        native_plugin_host: Option<NativePluginHostHandle>,
    ) -> Self {
        Self {
            resolved_config,
            module_selection_report,
            diagnostics,
            ownership: Some(ProductOwnership::new(
                runtime,
                plugin_bridge_lifecycle_state,
                compiled_project_plugin_plan,
                native_plugin_host,
            )),
        }
    }

    /// Returns the admitted product configuration used for this generation.
    pub const fn resolved_config(&self) -> &ResolvedProductHostConfig {
        &self.resolved_config
    }

    /// Borrows the bootstrapped Core inside the App-owned host boundary.
    pub(crate) fn core(&self) -> &CoreHandle {
        &self.ownership().core
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
        self.ownership().compiled_project_plugin_plan.as_deref()
    }

    /// Returns the retained runtime plugin bridge lifecycle state, when present.
    pub fn runtime_plugin_bridge_lifecycle_state(
        &self,
    ) -> Option<&RuntimePluginBridgeLifecycleState> {
        self.ownership().plugin_bridge_lifecycle_state.as_ref()
    }

    /// Returns the live native plugin host owner, when native discovery was requested.
    pub fn native_plugin_host(&self) -> Option<&NativePluginHostHandle> {
        self.ownership().native_plugin_host.as_ref()
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
        self.ownership().native_plugin_host.as_ref().ok_or_else(|| {
            "product composition does not own a native plugin host for this generation".to_owned()
        })
    }
}

#[derive(Debug)]
struct UnclosedProductComposition;
impl std::fmt::Display for UnclosedProductComposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("product composition dropped without an explicit close receipt")
    }
}
impl Error for UnclosedProductComposition {}

impl Drop for ProductComposition {
    fn drop(&mut self) {
        if let Some(product) = self.ownership.take() {
            // Retention only: Drop cannot call user cleanup, destroy a DLL or report Joined.
            let _ = ProductCompositionFailure::retained(
                Arc::new(UnclosedProductComposition),
                RetainedPacket::product(product),
            );
        }
    }
}
