// 扫描中的功能及子表暂存值；最终由表切换逻辑归并为可比较的静态签名。
use super::{OptionalFeatureDependencySignature, OptionalFeatureModuleSignature};

#[derive(Default)]
pub(in crate::tests::optional_feature_manifest::support) struct PendingOptionalFeatureManifest {
    pub(in crate::tests::optional_feature_manifest::support) id: Option<String>,
    pub(in crate::tests::optional_feature_manifest::support) display_name: Option<String>,
    pub(in crate::tests::optional_feature_manifest::support) owner_plugin_id: Option<String>,
    pub(in crate::tests::optional_feature_manifest::support) capabilities: Vec<String>,
    pub(in crate::tests::optional_feature_manifest::support) default_packaging:
        Vec<zircon_runtime::core::framework::project::ExportPackagingStrategy>,
    pub(in crate::tests::optional_feature_manifest::support) enabled_by_default: Option<bool>,
    pub(in crate::tests::optional_feature_manifest::support) dependencies:
        Vec<OptionalFeatureDependencySignature>,
    pub(in crate::tests::optional_feature_manifest::support) modules:
        Vec<OptionalFeatureModuleSignature>,
}
