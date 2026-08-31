pub(in crate::ui::retained_host::app::module_plugin_actions) fn feature_dependency_enable_message(
    report: &crate::ui::host::EditorPluginFeatureSelectionUpdateReport,
) -> String {
    let dependencies_enabled = !report.enabled_dependency_plugins.is_empty()
        || !report.enabled_dependency_features.is_empty();
    let mut message = String::with_capacity(feature_dependency_message_capacity(report));
    message.push_str("Feature ");
    message.push_str(&report.feature_id);
    if !dependencies_enabled {
        message.push_str(" dependencies already enabled");
        if !report.diagnostics.is_empty() {
            message.push_str(": ");
            push_joined(&mut message, &report.diagnostics, "; ");
        }
        return message;
    }

    message.push_str(" dependencies enabled: ");
    if !report.enabled_dependency_plugins.is_empty() {
        message.push_str("plugins ");
        push_joined(&mut message, &report.enabled_dependency_plugins, ", ");
    }
    if !report.enabled_dependency_features.is_empty() {
        if !report.enabled_dependency_plugins.is_empty() {
            message.push_str("; ");
        }
        message.push_str("features ");
        push_joined(&mut message, &report.enabled_dependency_features, ", ");
    }
    if !report.diagnostics.is_empty() {
        message.push_str("; ");
        push_joined(&mut message, &report.diagnostics, "; ");
    }
    message
}

fn feature_dependency_message_capacity(
    report: &crate::ui::host::EditorPluginFeatureSelectionUpdateReport,
) -> usize {
    let dependencies_enabled = !report.enabled_dependency_plugins.is_empty()
        || !report.enabled_dependency_features.is_empty();
    let mut capacity = "Feature ".len().saturating_add(report.feature_id.len());
    if !dependencies_enabled {
        capacity = capacity.saturating_add(" dependencies already enabled".len());
        if !report.diagnostics.is_empty() {
            capacity = capacity
                .saturating_add(": ".len())
                .saturating_add(joined_text_len(&report.diagnostics, "; ".len()));
        }
        return capacity;
    }

    capacity = capacity.saturating_add(" dependencies enabled: ".len());
    if !report.enabled_dependency_plugins.is_empty() {
        capacity = capacity
            .saturating_add("plugins ".len())
            .saturating_add(joined_text_len(
                &report.enabled_dependency_plugins,
                ", ".len(),
            ));
    }
    if !report.enabled_dependency_features.is_empty() {
        if !report.enabled_dependency_plugins.is_empty() {
            capacity = capacity.saturating_add("; ".len());
        }
        capacity = capacity
            .saturating_add("features ".len())
            .saturating_add(joined_text_len(
                &report.enabled_dependency_features,
                ", ".len(),
            ));
    }
    if !report.diagnostics.is_empty() {
        capacity = capacity
            .saturating_add("; ".len())
            .saturating_add(joined_text_len(&report.diagnostics, "; ".len()));
    }
    capacity
}

fn joined_text_len(values: &[String], separator_len: usize) -> usize {
    values
        .iter()
        .fold(0usize, |length, value| length.saturating_add(value.len()))
        .saturating_add(values.len().saturating_sub(1).saturating_mul(separator_len))
}

fn push_joined(output: &mut String, values: &[String], separator: &str) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(separator);
        }
        output.push_str(value);
    }
}

#[cfg(test)]
#[path = "dependencies/capacity_tests.rs"]
mod capacity_tests;
