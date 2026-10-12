use super::super::super::types::OptionalFeatureModuleSignature;

pub(in super::super) fn module_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<OptionalFeatureModuleSignature> {
    let mut modules = super::projection::project_module_signatures(feature);
    super::ordering::sort_module_signatures(&mut modules);
    modules
}
