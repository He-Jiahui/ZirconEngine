// 已格式化的失败原因同时保留在普通诊断和 fatal 列表，供调用方展示并阻止把失败报告判为成功。
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn push_fatal_diagnostic(
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
    diagnostic: String,
) {
    diagnostics.push(diagnostic.clone());
    fatal_diagnostics.push(diagnostic);
}
