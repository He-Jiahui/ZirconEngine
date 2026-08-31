use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn feature_project_selection_target_modes(
    feature: &PluginFeatureBundleManifest,
) -> Vec<RuntimeTargetMode> {
    let mut target_modes = Vec::new();
    let mut seen_target_modes = 0_u8;
    for target_mode in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        let bit = feature_target_mode_bit(target_mode);
        if seen_target_modes & bit == 0 {
            seen_target_modes |= bit;
            target_modes.push(target_mode);
        }
    }
    target_modes
}

const fn feature_target_mode_bit(target_mode: RuntimeTargetMode) -> u8 {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => 0b001,
        RuntimeTargetMode::ServerRuntime => 0b010,
        RuntimeTargetMode::EditorHost => 0b100,
    }
}

#[cfg(test)]
#[path = "target_modes/bitset_tests.rs"]
mod bitset_tests;
