mod uniqueness;

use super::path::validate_runtime_plugin_package_root;

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
