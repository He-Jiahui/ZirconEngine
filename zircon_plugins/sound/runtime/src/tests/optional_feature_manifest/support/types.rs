pub(super) type OptionalFeatureDependencySignature = (String, String, bool);

pub(super) type OptionalFeatureModuleSignature = (
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::RuntimeTargetMode>,
    Vec<String>,
);

#[derive(Debug, PartialEq, Eq)]
pub(in crate::tests::optional_feature_manifest) struct StaticOptionalFeatureManifest {
    pub(super) id: String,
    pub(super) display_name: String,
    pub(super) owner_plugin_id: String,
    pub(super) capabilities: Vec<String>,
    pub(super) default_packaging: Vec<zircon_runtime::plugin::ExportPackagingStrategy>,
    pub(super) enabled_by_default: bool,
    pub(super) dependencies: Vec<OptionalFeatureDependencySignature>,
    pub(super) modules: Vec<OptionalFeatureModuleSignature>,
}

impl StaticOptionalFeatureManifest {
    pub(in crate::tests::optional_feature_manifest) fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Default)]
pub(super) struct PendingOptionalFeatureManifest {
    pub(super) id: Option<String>,
    pub(super) display_name: Option<String>,
    pub(super) owner_plugin_id: Option<String>,
    pub(super) capabilities: Vec<String>,
    pub(super) default_packaging: Vec<zircon_runtime::plugin::ExportPackagingStrategy>,
    pub(super) enabled_by_default: Option<bool>,
    pub(super) dependencies: Vec<OptionalFeatureDependencySignature>,
    pub(super) modules: Vec<OptionalFeatureModuleSignature>,
}
