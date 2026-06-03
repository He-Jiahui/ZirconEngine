use super::super::feature_report::RuntimePluginFeatureDependencyReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn append_feature_dependency_diagnostics(
    feature_report: &RuntimePluginFeatureDependencyReport,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    diagnostics.extend(feature_report.diagnostics.iter().cloned());
    fatal_diagnostics.extend(feature_report.diagnostics.iter().cloned());
    for blocked in &feature_report.blocked_features {
        let diagnostic = blocked.to_diagnostic();
        if blocked.required {
            fatal_diagnostics.push(diagnostic.clone());
        }
        diagnostics.push(diagnostic);
    }
}
