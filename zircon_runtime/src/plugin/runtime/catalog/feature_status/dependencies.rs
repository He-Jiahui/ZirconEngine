use std::collections::HashSet;

use crate::plugin::PluginFeatureBundleManifest;

use super::super::feature_status_record::FeatureStatus;
use super::super::feature_support::plugin_is_enabled_for_target;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn append_dependency_status(
    status: &mut FeatureStatus,
    feature: &PluginFeatureBundleManifest,
    selected_plugin_ids: &HashSet<String>,
    canonical_enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
) {
    for dependency in &feature.dependencies {
        if !plugin_is_enabled_for_target(
            &dependency.plugin_id,
            selected_plugin_ids,
            canonical_enabled_plugins,
        ) {
            status.add_missing_plugin(&dependency.plugin_id);
        }
        if !available_capabilities.contains(&dependency.capability) {
            status.add_missing_capability(&dependency.capability);
        }
    }
}
