mod dependencies;

use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginFeatureSelection, ProjectPluginSelection};

use super::feature_definitions::FeatureDefinition;
use super::feature_status_record::FeatureStatus;
use super::feature_support::{
    feature_manifest_supports_target, owner_dependency_is_valid, plugin_is_enabled_for_target,
};
use dependencies::append_dependency_status;

pub(super) fn feature_status(
    feature_definition: &FeatureDefinition,
    selection: &ProjectPluginFeatureSelection,
    provider_registration_present: bool,
    target: RuntimeTargetMode,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
) -> FeatureStatus {
    let feature = &feature_definition.manifest;
    let mut status = FeatureStatus::new(feature.id.clone(), feature.owner_plugin_id.clone());
    if !provider_registration_present {
        status.mark_provider_missing();
    }
    if !owner_dependency_is_valid(feature) {
        status.mark_invalid_owner_dependency();
    }
    if !plugin_is_enabled_for_target(&feature.owner_plugin_id, plugin_selections, enabled_plugins) {
        status.add_missing_plugin(&feature.owner_plugin_id);
    }
    if feature_definition.provider_package_id != feature.owner_plugin_id
        && !plugin_is_enabled_for_target(
            &feature_definition.provider_package_id,
            plugin_selections,
            enabled_plugins,
        )
    {
        status.add_missing_plugin(&feature_definition.provider_package_id);
    }
    if !feature_manifest_supports_target(feature, target) || !selection.supports_target(target) {
        status.mark_target_unsupported();
    }
    append_dependency_status(
        &mut status,
        feature,
        plugin_selections,
        enabled_plugins,
        available_capabilities,
    );
    status
}
