use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::RuntimeExtensionRegistry;

use super::super::extension_merge::merge_feature_extensions_for_target;
use super::super::RuntimePluginFeatureRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_selected_feature_extensions(
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    selected_registration_indices: &[usize],
    target: RuntimeTargetMode,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for index in selected_registration_indices {
        merge_feature_extensions_for_target(
            &feature_registrations[*index],
            target,
            registry,
            diagnostics,
            fatal_diagnostics,
        );
    }
}
