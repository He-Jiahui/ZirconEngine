use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn feature_project_selection_target_modes(
    feature: &PluginFeatureBundleManifest,
) -> Vec<RuntimeTargetMode> {
    let mut target_modes = Vec::new();
    for target_mode in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    target_modes
}
