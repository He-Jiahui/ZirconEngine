use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitive;

#[derive(Clone, Debug, PartialEq)]
pub enum PanePayload {
    ConsoleV1(ConsolePanePayload),
    InspectorV1(InspectorPanePayload),
    HierarchyV1(HierarchyPanePayload),
    AnimationSequenceV1(AnimationSequencePanePayload),
    AnimationGraphV1(AnimationGraphPanePayload),
    RuntimeDiagnosticsV1(RuntimeDiagnosticsPanePayload),
    ModulePluginsV1(ModulePluginsPanePayload),
    BuildExportV1(BuildExportPanePayload),
    UiComponentShowcaseV1(UiComponentShowcasePanePayload),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsolePanePayload {
    pub status_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorPanePayload {
    pub node_id: u64,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
    pub delete_enabled: bool,
    pub plugin_components: Vec<InspectorPluginComponentPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorPluginComponentPayload {
    pub component_id: String,
    pub display_name: String,
    pub plugin_id: String,
    pub drawer_available: bool,
    pub drawer_ui_document: Option<String>,
    pub drawer_controller: Option<String>,
    pub drawer_template_id: Option<String>,
    pub drawer_data_root: Option<String>,
    pub drawer_bindings: Vec<String>,
    pub diagnostic: Option<String>,
    pub properties: Vec<InspectorPluginComponentPropertyPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorPluginComponentPropertyPayload {
    pub field_id: String,
    pub name: String,
    pub label: String,
    pub value: String,
    pub value_kind: String,
    pub editable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyPanePayload {
    pub nodes: Vec<HierarchyPaneNodePayload>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyPaneNodePayload {
    pub node_id: u64,
    pub name: String,
    pub depth: u32,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSequencePanePayload {
    pub mode: String,
    pub asset_path: String,
    pub status: String,
    pub selection: String,
    pub current_frame: u32,
    pub timeline_start_frame: u32,
    pub timeline_end_frame: u32,
    pub playback_label: String,
    pub track_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationGraphPanePayload {
    pub mode: String,
    pub asset_path: String,
    pub status: String,
    pub selection: String,
    pub parameter_items: Vec<String>,
    pub node_items: Vec<String>,
    pub state_items: Vec<String>,
    pub transition_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeDiagnosticsPanePayload {
    pub summary: String,
    pub render_status: String,
    pub physics_status: String,
    pub animation_status: String,
    pub detail_items: Vec<String>,
    pub ui_debug_reflector_summary: String,
    pub ui_debug_reflector_nodes: Vec<String>,
    pub ui_debug_reflector_details: Vec<String>,
    pub ui_debug_reflector_export_status: String,
    pub ui_debug_reflector_overlay_primitives: Vec<UiDebugOverlayPrimitive>,
    pub ui_debug_reflector_has_active_snapshot: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModulePluginsPanePayload {
    pub diagnostics: String,
    pub plugins: Vec<ModulePluginStatusPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModulePluginStatusPayload {
    pub plugin_id: String,
    pub display_name: String,
    pub package_source: String,
    pub load_state: String,
    pub enabled: bool,
    pub required: bool,
    pub target_modes: String,
    pub packaging: String,
    pub runtime_crate: String,
    pub editor_crate: String,
    pub runtime_capabilities: String,
    pub editor_capabilities: String,
    pub optional_features: String,
    pub feature_action_label: String,
    pub feature_action_id: String,
    pub diagnostics: String,
    pub primary_action_label: String,
    pub primary_action_id: String,
    pub packaging_action_label: String,
    pub packaging_action_id: String,
    pub target_modes_action_label: String,
    pub target_modes_action_id: String,
    pub unload_action_label: String,
    pub unload_action_id: String,
    pub hot_reload_action_label: String,
    pub hot_reload_action_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildExportPanePayload {
    pub diagnostics: String,
    pub targets: Vec<BuildExportTargetPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildExportTargetPayload {
    pub profile_name: String,
    pub platform: String,
    pub target_mode: String,
    pub strategies: String,
    pub status: String,
    pub enabled_plugins: String,
    pub linked_runtime_crates: String,
    pub native_dynamic_packages: String,
    pub generated_files: String,
    pub diagnostics: String,
    pub fatal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiComponentShowcasePanePayload {
    pub state_summary: String,
}
