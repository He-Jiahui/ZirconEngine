use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct ModulePluginStatusData {
    pub plugin_id: SharedString,
    pub display_name: SharedString,
    pub package_source: SharedString,
    pub load_state: SharedString,
    pub enabled: bool,
    pub required: bool,
    pub target_modes: SharedString,
    pub packaging: SharedString,
    pub runtime_crate: SharedString,
    pub editor_crate: SharedString,
    pub runtime_capabilities: SharedString,
    pub editor_capabilities: SharedString,
    pub optional_features: SharedString,
    pub feature_action_label: SharedString,
    pub feature_action_id: SharedString,
    pub diagnostics: SharedString,
    pub primary_action_label: SharedString,
    pub primary_action_id: SharedString,
    pub packaging_action_label: SharedString,
    pub packaging_action_id: SharedString,
    pub target_modes_action_label: SharedString,
    pub target_modes_action_id: SharedString,
    pub unload_action_label: SharedString,
    pub unload_action_id: SharedString,
    pub hot_reload_action_label: SharedString,
    pub hot_reload_action_id: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct ModulePluginsPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub plugins: ModelRc<ModulePluginStatusData>,
    pub diagnostics: SharedString,
}
