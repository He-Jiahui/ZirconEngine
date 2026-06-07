use super::super::super::types::OptionalFeatureModuleSignature;
use super::signature;

pub(super) fn project_module_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<OptionalFeatureModuleSignature> {
    feature
        .modules
        .iter()
        .map(signature::module_signature)
        .collect()
}
