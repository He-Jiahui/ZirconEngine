use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginStatus {
    pub plugin_id: String,
    pub display_name: String,
    pub package_source: String,
    pub package_metadata: Option<EditorPluginPackageMetadata>,
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

/// Metadata copied from the package already used to construct the owning status row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginPackageMetadata {
    pub version: String,
    pub category: String,
    pub description: String,
    pub maturity: zircon_runtime::plugin::PluginMaturity,
}

impl From<&zircon_runtime::plugin::PluginPackageManifest> for EditorPluginPackageMetadata {
    fn from(package: &zircon_runtime::plugin::PluginPackageManifest) -> Self {
        Self {
            version: package.version.clone(),
            category: package.category.clone(),
            description: package.description.clone(),
            maturity: package.maturity,
        }
    }
}
