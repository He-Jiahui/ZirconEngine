use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind};

pub(super) fn owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
    let primary_dependencies = feature
        .dependencies
        .iter()
        .filter(|dependency| dependency.primary)
        .collect::<Vec<_>>();
    primary_dependencies.len() == 1 && primary_dependencies[0].plugin_id == feature.owner_plugin_id
}

pub(super) fn plugin_is_enabled_for_target(
    plugin_id: &str,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
) -> bool {
    plugin_selections.contains_key(plugin_id) && enabled_plugins.contains(plugin_id)
}

pub(super) fn feature_manifest_supports_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> bool {
    let runtime_modules = feature
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .collect::<Vec<_>>();
    if runtime_modules.is_empty() {
        return true;
    }
    runtime_modules
        .iter()
        .any(|module| module.target_modes.is_empty() || module.target_modes.contains(&target))
}
