use super::types::StaticOptionalFeatureManifest;

pub(super) fn optional_feature_signature(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> StaticOptionalFeatureManifest {
    let mut capabilities = feature.capabilities.clone();
    let mut dependencies = feature
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.plugin_id.clone(),
                dependency.capability.clone(),
                dependency.primary,
            )
        })
        .collect::<Vec<_>>();
    let mut modules = feature
        .modules
        .iter()
        .map(|module| {
            (
                module.name.clone(),
                module.kind,
                module.crate_name.clone(),
                module.target_modes.clone(),
                module.capabilities.clone(),
            )
        })
        .collect::<Vec<_>>();
    capabilities.sort_unstable();
    dependencies.sort_unstable();
    modules.sort_unstable_by_key(|module| module.0.clone());

    StaticOptionalFeatureManifest {
        id: feature.id.clone(),
        display_name: feature.display_name.clone(),
        owner_plugin_id: feature.owner_plugin_id.clone(),
        capabilities,
        default_packaging: feature.default_packaging.clone(),
        enabled_by_default: feature.enabled_by_default,
        dependencies,
        modules,
    }
}
