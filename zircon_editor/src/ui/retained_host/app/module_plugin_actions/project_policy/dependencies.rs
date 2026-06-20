pub(in crate::ui::retained_host::app::module_plugin_actions) fn feature_dependency_enable_message(
    report: &crate::ui::host::EditorPluginFeatureSelectionUpdateReport,
) -> String {
    let mut details = Vec::new();
    if !report.enabled_dependency_plugins.is_empty() {
        details.push(format!(
            "plugins {}",
            report.enabled_dependency_plugins.join(", ")
        ));
    }
    if !report.enabled_dependency_features.is_empty() {
        details.push(format!(
            "features {}",
            report.enabled_dependency_features.join(", ")
        ));
    }
    if details.is_empty() {
        let mut message = format!("Feature {} dependencies already enabled", report.feature_id);
        if !report.diagnostics.is_empty() {
            message.push_str(": ");
            message.push_str(&report.diagnostics.join("; "));
        }
        return message;
    }
    let mut message = format!(
        "Feature {} dependencies enabled: {}",
        report.feature_id,
        details.join("; ")
    );
    if !report.diagnostics.is_empty() {
        message.push_str("; ");
        message.push_str(&report.diagnostics.join("; "));
    }
    message
}
