use super::super::pane_payload::{
    ModulePluginDetailsPayload, ModulePluginFeatureRowPayload, ModulePluginStatusPayload,
    ModulePluginsPanePayload, PanePayload,
};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let Some(data) = context.module_plugins else {
        return PanePayload::ModulePluginsV1(ModulePluginsPanePayload {
            diagnostics: String::new(),
            plugins: Vec::new(),
            detail_generation: 0,
            detail_locale: String::new(),
            selected_plugin_id: String::new(),
            focused_control: String::new(),
            details: None,
        });
    };
    let plugins = data
        .plugins
        .iter()
        .map(|plugin| ModulePluginStatusPayload {
            plugin_id: plugin.plugin_id.to_string(),
            display_name: plugin.display_name.to_string(),
            package_source: plugin.package_source.to_string(),
            package_metadata: plugin.package_metadata.clone(),
            load_state: plugin.load_state.to_string(),
            enabled: plugin.enabled,
            required: plugin.required,
            target_modes: plugin.target_modes.to_string(),
            packaging: plugin.packaging.to_string(),
            runtime_crate: plugin.runtime_crate.to_string(),
            editor_crate: plugin.editor_crate.to_string(),
            runtime_capabilities: plugin.runtime_capabilities.to_string(),
            editor_capabilities: plugin.editor_capabilities.to_string(),
            optional_features: plugin.optional_features.to_string(),
            feature_action_label: plugin.feature_action_label.to_string(),
            feature_action_id: plugin.feature_action_id.to_string(),
            diagnostics: plugin.diagnostics.to_string(),
            primary_action_label: plugin.primary_action_label.to_string(),
            primary_action_id: plugin.primary_action_id.to_string(),
            packaging_action_label: plugin.packaging_action_label.to_string(),
            packaging_action_id: plugin.packaging_action_id.to_string(),
            target_modes_action_label: plugin.target_modes_action_label.to_string(),
            target_modes_action_id: plugin.target_modes_action_id.to_string(),
            unload_action_label: plugin.unload_action_label.to_string(),
            unload_action_id: plugin.unload_action_id.to_string(),
            hot_reload_action_label: plugin.hot_reload_action_label.to_string(),
            hot_reload_action_id: plugin.hot_reload_action_id.to_string(),
        })
        .collect();

    PanePayload::ModulePluginsV1(ModulePluginsPanePayload {
        diagnostics: data.diagnostics.to_string(),
        plugins,
        detail_generation: data.detail_generation,
        detail_locale: data.detail_locale.to_string(),
        selected_plugin_id: data.selected_plugin_id.to_string(),
        focused_control: data.focused_control.to_string(),
        details: data
            .details
            .as_deref()
            .map(|detail| ModulePluginDetailsPayload {
                plugin_id: detail.plugin_id.clone(),
                title: detail.title.clone(),
                page: detail.page,
                total_features: detail.total_features,
                rows: detail
                    .rows
                    .iter()
                    .map(|row| ModulePluginFeatureRowPayload {
                        feature_id: row.feature_id.clone(),
                        label: row.label.clone(),
                        desired: row.desired,
                        required: row.required,
                        blocked: row.blocked,
                        effective: row.effective.clone(),
                    })
                    .collect(),
                selected_feature_id: detail.selected_feature_id.clone(),
                selected_identity: detail.selected_identity.clone(),
                selected_name: detail.selected_name.clone(),
                selected_states: detail.selected_states.clone(),
                selected_reason: detail.selected_reason.clone(),
                action_label: detail.action_label.clone(),
                action_id: detail.action_id.clone(),
                intent_id: detail.intent_id.clone(),
                back_label: detail.back_label.clone(),
                previous_label: detail.previous_label.clone(),
                next_label: detail.next_label.clone(),
                page_label: detail.page_label.clone(),
                empty_label: detail.empty_label.clone(),
            }),
    })
}
