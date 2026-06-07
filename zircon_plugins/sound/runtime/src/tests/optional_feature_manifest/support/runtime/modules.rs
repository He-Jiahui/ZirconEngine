mod ordering;
mod projection;
mod signature;

use super::super::types::OptionalFeatureModuleSignature;

pub(super) fn module_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<OptionalFeatureModuleSignature> {
    let mut modules = projection::project_module_signatures(feature);
    ordering::sort_module_signatures(&mut modules);
    modules
}
