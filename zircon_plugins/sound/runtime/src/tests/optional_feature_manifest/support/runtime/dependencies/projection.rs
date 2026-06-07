use super::super::super::types::OptionalFeatureDependencySignature;
use super::signature;

pub(super) fn project_dependency_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<OptionalFeatureDependencySignature> {
    feature
        .dependencies
        .iter()
        .map(signature::dependency_signature)
        .collect()
}
