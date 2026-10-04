use super::super::feature_report::RuntimePluginFeatureDependencyReport;

// 所有特性阻塞都进入普通诊断；required 阻塞再进入 fatal 列表，供调用方根据 fatal 状态判断计划是否可运行。
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
