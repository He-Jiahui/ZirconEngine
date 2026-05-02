use zircon_runtime::{plugin::ExportPackagingStrategy, RuntimeTargetMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginStatus {
    pub plugin_id: String,
    pub display_name: String,
    pub package_source: String,
    pub load_state: String,
    pub enabled: bool,
    pub required: bool,
    pub target_modes: Vec<RuntimeTargetMode>,
    pub packaging: ExportPackagingStrategy,
    pub runtime_crate: Option<String>,
    pub editor_crate: Option<String>,
    pub runtime_capabilities: Vec<String>,
    pub editor_capabilities: Vec<String>,
    pub optional_features: Vec<EditorPluginFeatureStatus>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginFeatureStatus {
    pub id: String,
    pub display_name: String,
    pub owner_plugin_id: String,
    pub enabled: bool,
    pub required: bool,
    pub available: bool,
    pub target_modes: Vec<RuntimeTargetMode>,
    pub packaging: ExportPackagingStrategy,
    pub runtime_crate: Option<String>,
    pub editor_crate: Option<String>,
    pub provided_capabilities: Vec<String>,
    pub dependencies: Vec<EditorPluginFeatureDependencyStatus>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginFeatureDependencyStatus {
    pub plugin_id: String,
    pub capability: String,
    pub primary: bool,
    pub plugin_enabled: bool,
    pub capability_available: bool,
}
