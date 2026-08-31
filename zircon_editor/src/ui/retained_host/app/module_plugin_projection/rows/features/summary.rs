use std::fmt::Write;

use crate::ui::host::{EditorPluginFeatureDependencyStatus, EditorPluginFeatureStatus};

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_optional_feature_summary(
    features: &[EditorPluginFeatureStatus],
) -> String {
    let mut summary = String::with_capacity(module_plugin_feature_summary_capacity(features));
    for (feature_index, feature) in features.iter().enumerate() {
        if feature_index != 0 {
            summary.push('\n');
        }
        let state = module_plugin_feature_state(feature);
        write!(&mut summary, "{} [{state}]", feature.display_name)
            .expect("writing to String cannot fail");
        if !feature.dependencies.is_empty() {
            summary.push_str(" deps: ");
            push_module_plugin_feature_dependencies(&mut summary, &feature.dependencies);
        }
    }
    summary
}

fn module_plugin_feature_state(feature: &EditorPluginFeatureStatus) -> &'static str {
    if feature.enabled {
        if feature.available {
            "enabled"
        } else {
            "blocked"
        }
    } else if feature.available {
        "ready"
    } else {
        "blocked"
    }
}

fn push_module_plugin_feature_dependencies(
    summary: &mut String,
    dependencies: &[EditorPluginFeatureDependencyStatus],
) {
    for (dependency_index, dependency) in dependencies.iter().enumerate() {
        if dependency_index != 0 {
            summary.push_str("; ");
        }
        let dependency_state = module_plugin_feature_dependency_state(dependency);
        let role = if dependency.primary { "primary " } else { "" };
        write!(
            summary,
            "{role}{}:{} ({dependency_state})",
            dependency.plugin_id, dependency.capability
        )
        .expect("writing to String cannot fail");
    }
}

fn module_plugin_feature_dependency_state(
    dependency: &EditorPluginFeatureDependencyStatus,
) -> &'static str {
    match (dependency.plugin_enabled, dependency.capability_available) {
        (true, true) => "ok",
        (false, _) => "missing plugin",
        (true, false) => "missing capability",
    }
}

fn module_plugin_feature_summary_capacity(features: &[EditorPluginFeatureStatus]) -> usize {
    features
        .iter()
        .enumerate()
        .map(|(feature_index, feature)| {
            let mut capacity = usize::from(feature_index != 0)
                + feature.display_name.len()
                + module_plugin_feature_state(feature).len()
                + 3;
            if !feature.dependencies.is_empty() {
                capacity += 7;
                for (dependency_index, dependency) in feature.dependencies.iter().enumerate() {
                    capacity += usize::from(dependency_index != 0) * 2
                        + usize::from(dependency.primary) * "primary ".len()
                        + dependency.plugin_id.len()
                        + dependency.capability.len()
                        + module_plugin_feature_dependency_state(dependency).len()
                        + 4;
                }
            }
            capacity
        })
        .sum()
}

#[cfg(test)]
#[path = "summary/single_buffer_tests.rs"]
mod single_buffer_tests;
