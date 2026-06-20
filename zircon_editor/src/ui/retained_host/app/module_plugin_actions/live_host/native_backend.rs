use std::sync::Arc;

use zircon_runtime::plugin::native::NativePluginLiveHost;

use super::types::{
    ModulePluginLiveHostBackend, ModulePluginLiveHostCommand, ModulePluginLiveHostOutcome,
    ModulePluginLiveHostRequest,
};

impl ModulePluginLiveHostBackend for NativePluginLiveHost {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        let outcome = match request.command {
            ModulePluginLiveHostCommand::Unload => self.unload_editor_plugin(request.plugin_id),
            ModulePluginLiveHostCommand::HotReload => {
                self.hot_reload_editor_plugin(request.project_root, request.plugin_id)
            }
        }?;
        Ok(ModulePluginLiveHostOutcome {
            plugin_id: outcome.plugin_id,
            command: request.command,
            diagnostics: outcome.diagnostics,
        })
    }
}

impl ModulePluginLiveHostBackend for Arc<NativePluginLiveHost> {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        self.as_ref().execute(request)
    }
}
