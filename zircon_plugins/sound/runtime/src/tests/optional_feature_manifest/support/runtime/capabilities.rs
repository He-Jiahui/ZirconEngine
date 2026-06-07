mod ordering;
mod projection;

pub(super) fn capability_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<String> {
    let mut capabilities = projection::project_capability_signatures(feature);
    ordering::sort_capability_signatures(&mut capabilities);
    capabilities
}
