mod uniqueness;

use super::path::validate_runtime_plugin_package_root;

// 即使根路径无效，也保留重复诊断；注册报告一次呈现集合与路径两类问题。
pub(super) fn validate_runtime_plugin_package_root_array(
    field_name: &str,
    roots: &[String],
    is_duplicate: impl Fn(usize) -> bool,
    diagnostics: &mut Vec<String>,
) {
    for (index, root) in roots.iter().enumerate() {
        uniqueness::validate_runtime_plugin_package_root_uniqueness(
            field_name,
            root,
            is_duplicate(index),
            diagnostics,
        );
        validate_runtime_plugin_package_root(field_name, root, diagnostics);
    }
}
