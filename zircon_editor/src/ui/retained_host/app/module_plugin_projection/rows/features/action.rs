use crate::ui::host::EditorPluginFeatureStatus;

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_feature_action(
    features: &[EditorPluginFeatureStatus],
) -> (String, String) {
    let mut enable_feature = None;
    let mut disable_feature = None;
    for feature in features {
        if !feature.enabled && !feature.available {
            return (
                "Enable Deps".to_string(),
                module_plugin_feature_action_id(
                    "workbench.plugin.feature.enable_dependencies",
                    &feature.owner_plugin_id,
                    &feature.id,
                ),
            );
        }
        if !feature.enabled && enable_feature.is_none() {
            enable_feature = Some(feature);
        } else if feature.enabled && !feature.required && disable_feature.is_none() {
            disable_feature = Some(feature);
        }
    }
    if let Some(feature) = enable_feature {
        return (
            "Enable Feature".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.enable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = disable_feature {
        return (
            "Disable Feature".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.disable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    (String::new(), String::new())
}

fn module_plugin_feature_action_id(prefix: &str, plugin_id: &str, feature_id: &str) -> String {
    format!("{prefix}.{plugin_id}.{feature_id}")
}

#[cfg(test)]
#[path = "action/single_pass_priority_tests.rs"]
mod single_pass_priority_tests;
