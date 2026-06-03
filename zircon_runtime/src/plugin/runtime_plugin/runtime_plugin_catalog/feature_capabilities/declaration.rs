use crate::RuntimeTargetMode;

use super::super::feature_definitions::FeatureDefinition;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_declares_capability_for_target(
    feature_definition: &FeatureDefinition,
    capability: &str,
    target: RuntimeTargetMode,
) -> bool {
    let feature = &feature_definition.manifest;
    feature
        .capabilities
        .iter()
        .any(|provided| provided == capability)
        || feature
            .modules
            .iter()
            .filter(move |module| {
                module.target_modes.is_empty() || module.target_modes.contains(&target)
            })
            .any(|module| {
                module
                    .capabilities
                    .iter()
                    .any(|provided| provided == capability)
            })
}
