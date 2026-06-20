use crate::ui::host::EditorPluginFeatureStatus;

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_optional_feature_summary(
    features: &[EditorPluginFeatureStatus],
) -> String {
    features
        .iter()
        .map(|feature| {
            let state = module_plugin_feature_state(feature);
            let dependencies = module_plugin_feature_dependency_summary(feature);
            if dependencies.is_empty() {
                format!("{} [{state}]", feature.display_name)
            } else {
                format!("{} [{state}] deps: {dependencies}", feature.display_name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

fn module_plugin_feature_dependency_summary(feature: &EditorPluginFeatureStatus) -> String {
    feature
        .dependencies
        .iter()
        .map(|dependency| {
            let dependency_state =
                match (dependency.plugin_enabled, dependency.capability_available) {
                    (true, true) => "ok",
                    (false, _) => "missing plugin",
                    (true, false) => "missing capability",
                };
            let role = if dependency.primary { "primary " } else { "" };
            format!(
                "{role}{}:{} ({dependency_state})",
                dependency.plugin_id, dependency.capability
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
