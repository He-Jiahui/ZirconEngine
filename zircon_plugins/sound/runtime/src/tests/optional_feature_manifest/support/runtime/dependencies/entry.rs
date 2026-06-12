use super::super::super::types::OptionalFeatureDependencySignature;

pub(in super::super) fn dependency_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<OptionalFeatureDependencySignature> {
    let mut dependencies = super::projection::project_dependency_signatures(feature);
    super::ordering::sort_dependency_signatures(&mut dependencies);
    dependencies
}
