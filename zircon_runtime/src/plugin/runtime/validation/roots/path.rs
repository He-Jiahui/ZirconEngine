//! 包根路径声明先做字段、相对性和路径段检查；诊断进入注册报告，实际文件访问仍需独立的包边界验证。
#[path = "path/field.rs"]
mod field;
#[path = "path/relative.rs"]
mod relative;
#[path = "path/segments.rs"]
mod segments;
#[path = "path/separator.rs"]
mod separator;

pub(super) fn validate_runtime_plugin_package_root(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if !field::validate_runtime_plugin_package_root_field(field_name, root, diagnostics) {
        return;
    }
    relative::validate_runtime_plugin_package_root_relative(field_name, root, diagnostics);
    separator::validate_runtime_plugin_package_root_separator(field_name, root, diagnostics);
    segments::validate_runtime_plugin_package_root_segments(field_name, root, diagnostics);
}
