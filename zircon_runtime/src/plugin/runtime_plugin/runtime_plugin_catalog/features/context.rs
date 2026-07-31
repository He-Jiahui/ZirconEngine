use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_definitions::FeatureDefinitionMap;
use super::super::feature_report::RuntimePluginFeatureDependencyReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct FeatureDependencyContext<'a> {
    pub plugin_selections: HashMap<&'a str, &'a ProjectPluginSelection>,
    pub enabled_plugins: HashSet<String>,
    pub available_capabilities: HashSet<String>,
    pub report: RuntimePluginFeatureDependencyReport,
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_dependency_context<'a>(
    projection: &RuntimePluginCatalogProjection,
    completed: &'a ProjectPluginManifest,
    target: RuntimeTargetMode,
    feature_definitions: &FeatureDefinitionMap,
) -> FeatureDependencyContext<'a> {
    let plugin_selections = completed
        .selections
        .iter()
        .map(|selection| (selection.id.as_str(), selection))
        .collect::<HashMap<_, _>>();
    let enabled_plugins = completed
        .enabled_for_target(target)
        .map(|selection| selection.id.clone())
        .collect::<HashSet<_>>();
    let available_capabilities = projection.base_capabilities_for_target(&enabled_plugins, target);
    let report = RuntimePluginFeatureDependencyReport {
        diagnostics: feature_definitions.diagnostics.clone(),
        ..RuntimePluginFeatureDependencyReport::default()
    };
    FeatureDependencyContext {
        plugin_selections,
        enabled_plugins,
        available_capabilities,
        report,
    }
}
