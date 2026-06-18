use std::collections::{HashMap, HashSet};

use crate::builtin::RuntimeTargetMode;
use crate::plugin::{ProjectPluginManifest, ProjectPluginSelection};

use super::super::feature_capabilities::base_capabilities_for_target;
use super::super::feature_definitions::FeatureDefinitionMap;
use super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::RuntimePluginRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct FeatureDependencyContext<'a> {
    pub plugin_selections: HashMap<&'a str, &'a ProjectPluginSelection>,
    pub enabled_plugins: HashSet<String>,
    pub available_capabilities: HashSet<String>,
    pub report: RuntimePluginFeatureDependencyReport,
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_dependency_context<'a>(
    registrations: &[RuntimePluginRegistrationReport],
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
    let available_capabilities =
        base_capabilities_for_target(registrations, &enabled_plugins, target);
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
