use super::{OptionalFeatureDependencySignature, OptionalFeatureModuleSignature};

#[derive(Debug, PartialEq, Eq)]
pub(in crate::tests::optional_feature_manifest) struct StaticOptionalFeatureManifest {
    pub(in crate::tests::optional_feature_manifest::support) id: String,
    pub(in crate::tests::optional_feature_manifest::support) display_name: String,
    pub(in crate::tests::optional_feature_manifest::support) owner_plugin_id: String,
    pub(in crate::tests::optional_feature_manifest::support) capabilities: Vec<String>,
    pub(in crate::tests::optional_feature_manifest::support) default_packaging:
        Vec<zircon_runtime::plugin::ExportPackagingStrategy>,
    pub(in crate::tests::optional_feature_manifest::support) enabled_by_default: bool,
    pub(in crate::tests::optional_feature_manifest::support) dependencies:
        Vec<OptionalFeatureDependencySignature>,
    pub(in crate::tests::optional_feature_manifest::support) modules:
        Vec<OptionalFeatureModuleSignature>,
}

impl StaticOptionalFeatureManifest {
    pub(in crate::tests::optional_feature_manifest) fn id(&self) -> &str {
        &self.id
    }
}
