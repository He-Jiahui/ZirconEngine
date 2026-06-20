use crate::ui::host::{EditorPluginStatus, EditorPluginStatusReport};
use crate::ui::layouts::windows::workbench_host_window::ModulePluginStatusViewData;

use super::super::rows::{
    module_plugin_action_id, module_plugin_feature_action, module_plugin_optional_feature_summary,
    module_plugin_primary_action, packaging_label, target_mode_label,
};

pub(super) fn module_plugin_status_rows(
    report: EditorPluginStatusReport,
) -> Vec<ModulePluginStatusViewData> {
    report
        .plugins
        .into_iter()
        .map(module_plugin_status_row)
        .collect()
}

fn module_plugin_status_row(plugin: EditorPluginStatus) -> ModulePluginStatusViewData {
    let primary_action =
        module_plugin_primary_action(&plugin.plugin_id, plugin.enabled, plugin.required);
    let packaging_action_label = format!("Cycle {}", packaging_label(plugin.packaging));
    let packaging_action_id =
        module_plugin_action_id("workbench.plugin.packaging.next", &plugin.plugin_id);
    let target_modes_action_id =
        module_plugin_action_id("workbench.plugin.target_modes.next", &plugin.plugin_id);
    let unload_action_id = module_plugin_action_id("workbench.plugin.unload", &plugin.plugin_id);
    let hot_reload_action_id =
        module_plugin_action_id("workbench.plugin.hot_reload", &plugin.plugin_id);
    let feature_action = module_plugin_feature_action(&plugin.optional_features);

    ModulePluginStatusViewData {
        plugin_id: plugin.plugin_id.into(),
        display_name: plugin.display_name.into(),
        package_source: plugin.package_source.into(),
        load_state: plugin.load_state.into(),
        enabled: plugin.enabled,
        required: plugin.required,
        target_modes: plugin
            .target_modes
            .iter()
            .map(target_mode_label)
            .collect::<Vec<_>>()
            .join(", ")
            .into(),
        packaging: packaging_label(plugin.packaging).into(),
        runtime_crate: plugin.runtime_crate.unwrap_or_default().into(),
        editor_crate: plugin.editor_crate.unwrap_or_default().into(),
        runtime_capabilities: plugin.runtime_capabilities.join(", ").into(),
        editor_capabilities: plugin.editor_capabilities.join(", ").into(),
        optional_features: module_plugin_optional_feature_summary(&plugin.optional_features).into(),
        feature_action_label: feature_action.0.into(),
        feature_action_id: feature_action.1.into(),
        diagnostics: plugin.diagnostics.join("\n").into(),
        primary_action_label: primary_action.0.into(),
        primary_action_id: primary_action.1.into(),
        packaging_action_label: packaging_action_label.into(),
        packaging_action_id: packaging_action_id.into(),
        target_modes_action_label: "Cycle targets".into(),
        target_modes_action_id: target_modes_action_id.into(),
        unload_action_label: "Unload".into(),
        unload_action_id: unload_action_id.into(),
        hot_reload_action_label: "Hot Reload".into(),
        hot_reload_action_id: hot_reload_action_id.into(),
    }
}
