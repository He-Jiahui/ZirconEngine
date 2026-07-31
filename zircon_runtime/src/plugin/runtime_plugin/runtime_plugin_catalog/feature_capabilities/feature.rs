use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginFeatureBundleManifest;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_capabilities_for_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> impl Iterator<Item = &str> + '_ {
    feature.capabilities.iter().map(String::as_str).chain(
        feature
            .modules
            .iter()
            .filter(move |module| {
                module.target_modes.is_empty() || module.target_modes.contains(&target)
            })
            .flat_map(|module| module.capabilities.iter().map(String::as_str)),
    )
}
