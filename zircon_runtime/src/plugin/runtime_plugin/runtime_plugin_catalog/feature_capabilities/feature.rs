use crate::builtin::RuntimeTargetMode;
use crate::plugin::PluginFeatureBundleManifest;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_capabilities_for_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> impl Iterator<Item = String> + '_ {
    feature.capabilities.iter().cloned().chain(
        feature
            .modules
            .iter()
            .filter(move |module| {
                module.target_modes.is_empty() || module.target_modes.contains(&target)
            })
            .flat_map(|module| module.capabilities.iter().cloned()),
    )
}
