use std::path::Path;

use super::super::super::RetainedEditorHost;
use super::super::live_host::{
    ModulePluginLiveHostCommand, dispatch_live_plugin_backend_action,
    live_plugin_backend_success_message,
};

impl RetainedEditorHost {
    pub(super) fn dispatch_module_plugin_live_host_action(
        &self,
        plugin_id: &str,
        command: ModulePluginLiveHostCommand,
        project_root: &Path,
    ) -> Result<String, String> {
        let outcome = self
            .runtime
            .execute_native_live_action_without_active_contribution(plugin_id, || {
                dispatch_live_plugin_backend_action(
                    self.module_plugin_live_host_backend.as_ref(),
                    plugin_id,
                    command,
                    project_root,
                )
            })?;
        Ok(live_plugin_backend_success_message(&outcome))
    }
}
