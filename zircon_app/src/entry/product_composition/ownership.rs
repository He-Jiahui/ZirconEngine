use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::manager::resolve_manager_service;
use zircon_runtime::core::{CoreHandle, CoreRuntime, TaskGraphShutdownReport};
use zircon_runtime::plugin::native::host::NativePluginHostHandle;
use zircon_runtime::plugin::{CompiledProjectPluginPlan, RuntimePluginBridgeLifecycleState};

use crate::entry::product_shutdown::retained_owner::ProductCloseError;

#[derive(Debug)]
pub(in crate::entry) struct ProductOwnership {
    pub(super) runtime: CoreRuntime,
    pub(super) core: CoreHandle,
    pub(super) plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
    pub(super) compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
    pub(super) native_plugin_host: Option<NativePluginHostHandle>,
    project_watchers_shutdown: bool,
}

impl ProductOwnership {
    pub(super) fn new(
        runtime: CoreRuntime,
        plugin_bridge_lifecycle_state: Option<RuntimePluginBridgeLifecycleState>,
        compiled_project_plugin_plan: Option<Arc<CompiledProjectPluginPlan>>,
        native_plugin_host: Option<NativePluginHostHandle>,
    ) -> Self {
        Self {
            core: runtime.handle(),
            runtime,
            plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan,
            native_plugin_host,
            project_watchers_shutdown: false,
        }
    }

    pub(in crate::entry) fn close_until(
        &mut self,
        deadline: Instant,
    ) -> Result<TaskGraphShutdownReport, ProductCloseError> {
        if !self.project_watchers_shutdown {
            if let Ok(handle) = zircon_runtime::asset::project_asset_manager_handle(&self.core) {
                if let Ok(manager) = resolve_manager_service(&self.core, handle) {
                    if !manager.shutdown_project_watchers_until(deadline) {
                        return Err(ProductCloseError::ProjectWatchers);
                    }
                }
            }
            self.project_watchers_shutdown = true;
        }
        self.runtime
            .shutdown_until(deadline)
            .map_err(ProductCloseError::Core)
    }
}
