mod count;
mod tokens;

use self::{
    count::validate_runtime_plugin_feature_namespace_segment_count,
    tokens::validate_runtime_plugin_feature_namespace_segment_tokens,
};

// 先判命名空间是否具备分段结构；该结构缺失时只报这一原因，
// 具备结构后再审查每段内容，保留调用方所依赖的诊断顺序。
pub(super) fn validate_runtime_plugin_feature_namespace_segments(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !validate_runtime_plugin_feature_namespace_segment_count(field_name, value, diagnostics) {
        return;
    }
    validate_runtime_plugin_feature_namespace_segment_tokens(field_name, value, diagnostics);
}

#[cfg(test)]
#[path = "tests/segments.rs"]
mod tests;
