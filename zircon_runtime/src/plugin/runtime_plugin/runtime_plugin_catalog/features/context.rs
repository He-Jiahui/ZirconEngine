use std::collections::{HashMap, HashSet};

#[cfg(test)]
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

#[cfg(test)]
use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_definitions::FeatureDefinitionMap;
use super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::feature_support::{
    canonical_plugin_id, canonical_plugin_ids, canonical_plugin_selection_ids,
};

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct FeatureDependencyContext<'a> {
    pub plugin_selections: HashMap<&'a str, &'a ProjectPluginSelection>,
    pub selected_plugin_ids: HashSet<String>,
    pub enabled_plugins: HashSet<String>,
    pub canonical_enabled_plugins: HashSet<String>,
    pub available_capabilities: HashSet<String>,
    pub report: RuntimePluginFeatureDependencyReport,
}

#[cfg(test)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_dependency_context<'a>(
    projection: &RuntimePluginCatalogProjection,
    completed: &'a ProjectPluginManifest,
    target: RuntimeTargetMode,
    feature_definitions: &FeatureDefinitionMap,
) -> FeatureDependencyContext<'a> {
    let enabled_plugins = canonicalize_enabled_plugin_ids(
        completed
            .enabled_for_target(target)
            .map(|selection| selection.id.clone())
            .collect::<HashSet<_>>(),
    );
    let available_capabilities = projection.base_capabilities_for_target(&enabled_plugins, target);
    feature_dependency_context_for_effective_base(
        completed,
        feature_definitions,
        enabled_plugins,
        available_capabilities,
    )
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_dependency_context_for_effective_base<
    'a,
>(
    completed: &'a ProjectPluginManifest,
    feature_definitions: &FeatureDefinitionMap,
    enabled_plugins: HashSet<String>,
    available_capabilities: HashSet<String>,
) -> FeatureDependencyContext<'a> {
    let enabled_plugins = canonicalize_enabled_plugin_ids(enabled_plugins);
    let plugin_selections = completed
        .selections
        .iter()
        .map(|selection| (selection.id.as_str(), selection))
        .collect::<HashMap<_, _>>();
    let selected_plugin_ids = canonical_plugin_selection_ids(&plugin_selections);
    let canonical_enabled_plugins = canonical_plugin_ids(&enabled_plugins);
    let report = RuntimePluginFeatureDependencyReport {
        diagnostics: feature_definitions.diagnostics.clone(),
        ..RuntimePluginFeatureDependencyReport::default()
    };
    FeatureDependencyContext {
        plugin_selections,
        selected_plugin_ids,
        enabled_plugins,
        canonical_enabled_plugins,
        available_capabilities,
        report,
    }
}

/// Keep source-spelled IDs for diagnostics and manifest lookup, while adding canonical keys for
/// capability projection and feature owner/dependency predicates. This augments membership only;
/// it never inserts a second project-selection row under an alias.
fn canonicalize_enabled_plugin_ids(mut enabled_plugins: HashSet<String>) -> HashSet<String> {
    let canonical_ids = enabled_plugins
        .iter()
        .filter_map(|plugin_id| canonical_plugin_id(plugin_id))
        .collect::<Vec<_>>();
    enabled_plugins.extend(canonical_ids);
    enabled_plugins
}
