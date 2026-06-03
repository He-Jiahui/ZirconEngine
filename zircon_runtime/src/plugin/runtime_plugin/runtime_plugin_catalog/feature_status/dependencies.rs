use std::collections::{HashMap, HashSet};

use crate::plugin::{PluginFeatureBundleManifest, ProjectPluginSelection};

use super::super::feature_status_record::FeatureStatus;
use super::super::feature_support::plugin_is_enabled_for_target;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn append_dependency_status(
    status: &mut FeatureStatus,
    feature: &PluginFeatureBundleManifest,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
) {
    for dependency in &feature.dependencies {
        if !plugin_is_enabled_for_target(&dependency.plugin_id, plugin_selections, enabled_plugins)
        {
            status.add_missing_plugin(dependency.plugin_id.clone());
        }
        if !available_capabilities.contains(&dependency.capability) {
            status.add_missing_capability(dependency.capability.clone());
        }
    }
}
