use slint::Model;

use super::super::pane_payload::{
    ModulePluginStatusPayload, ModulePluginsPanePayload, PanePayload,
};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let data = context.module_plugins.cloned().unwrap_or_default();
    let plugins = (0..data.plugins.row_count())
        .filter_map(|row| data.plugins.row_data(row))
        .map(|plugin| ModulePluginStatusPayload {
            plugin_id: plugin.plugin_id.to_string(),
            display_name: plugin.display_name.to_string(),
            package_source: plugin.package_source.to_string(),
            load_state: plugin.load_state.to_string(),
            enabled: plugin.enabled,
            required: plugin.required,
            target_modes: plugin.target_modes.to_string(),
            packaging: plugin.packaging.to_string(),
            runtime_crate: plugin.runtime_crate.to_string(),
            editor_crate: plugin.editor_crate.to_string(),
            runtime_capabilities: plugin.runtime_capabilities.to_string(),
            editor_capabilities: plugin.editor_capabilities.to_string(),
            diagnostics: plugin.diagnostics.to_string(),
        })
        .collect();

    PanePayload::ModulePluginsV1(ModulePluginsPanePayload {
        diagnostics: data.diagnostics.to_string(),
        plugins,
    })
}
