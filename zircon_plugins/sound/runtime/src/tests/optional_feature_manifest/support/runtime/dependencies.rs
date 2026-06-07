mod ordering;
mod projection;
mod signature;

use super::super::types::OptionalFeatureDependencySignature;

pub(super) fn dependency_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<OptionalFeatureDependencySignature> {
    let mut dependencies = projection::project_dependency_signatures(feature);
    ordering::sort_dependency_signatures(&mut dependencies);
    dependencies
}
