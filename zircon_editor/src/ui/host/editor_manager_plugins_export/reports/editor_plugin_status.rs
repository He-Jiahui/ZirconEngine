use zircon_runtime::{ExportPackagingStrategy, RuntimeTargetMode};

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
    pub diagnostics: Vec<String>,
}
