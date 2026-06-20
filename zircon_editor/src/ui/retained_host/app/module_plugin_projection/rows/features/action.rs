use crate::ui::host::EditorPluginFeatureStatus;

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_feature_action(
    features: &[EditorPluginFeatureStatus],
) -> (String, String) {
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && !feature.available)
    {
        return (
            "Enable Deps".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.enable_dependencies",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && feature.available)
    {
        return (
            "Enable Feature".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.enable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| feature.enabled && !feature.required)
    {
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
