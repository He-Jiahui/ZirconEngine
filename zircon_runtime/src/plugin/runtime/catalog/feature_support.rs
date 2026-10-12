use std::collections::{HashMap, HashSet};

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind};

pub(super) fn canonical_plugin_id(raw: &str) -> Option<String> {
    RuntimePluginId::parse_key(raw).map(|id| id.key().to_owned())
}

pub(super) fn plugin_ids_match(left: &str, right: &str) -> bool {
    canonical_plugin_id(left)
        .zip(canonical_plugin_id(right))
        .is_some_and(|(left, right)| left == right)
}

// owner 依赖必须恰好有一个 primary 且 canonical ID 与 feature owner 相同；其余 required 依赖不参与这项校验。
pub(super) fn owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
    let mut primary_dependencies = feature
        .dependencies
        .iter()
        .filter(|dependency| dependency.primary);
    let Some(primary_dependency) = primary_dependencies.next() else {
        return false;
    };
    plugin_ids_match(&primary_dependency.plugin_id, &feature.owner_plugin_id)
        && primary_dependencies.next().is_none()
}

pub(super) fn plugin_is_enabled_for_target(
    plugin_id: &str,
    selected_plugin_ids: &HashSet<String>,
    enabled_plugin_ids: &HashSet<String>,
) -> bool {
    RuntimePluginId::parse_key(plugin_id).is_some_and(|canonical_id| {
        let canonical_key = canonical_id.key();
        selected_plugin_ids.contains(canonical_key) && enabled_plugin_ids.contains(canonical_key)
    })
}

pub(super) fn canonical_plugin_selection_ids(
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
) -> HashSet<String> {
    plugin_selections
        .keys()
        .filter_map(|selection_id| canonical_plugin_id(selection_id))
        .collect()
}

pub(super) fn canonical_plugin_ids(plugin_ids: &HashSet<String>) -> HashSet<String> {
    plugin_ids
        .iter()
        .filter_map(|plugin_id| canonical_plugin_id(plugin_id))
        .collect()
}

// 不含 Runtime 模块时不限制运行目标；包含时任一 Runtime 模块的目标为空或包含当前目标即可通过。
pub(super) fn feature_manifest_supports_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> bool {
    let mut runtime_modules = feature
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime);
    let Some(first_runtime_module) = runtime_modules.next() else {
        return true;
    };
    first_runtime_module.target_modes.is_empty()
        || first_runtime_module.target_modes.contains(&target)
        || runtime_modules
            .any(|module| module.target_modes.is_empty() || module.target_modes.contains(&target))
}

#[cfg(test)]
#[path = "tests/feature_support.rs"]
mod tests;
