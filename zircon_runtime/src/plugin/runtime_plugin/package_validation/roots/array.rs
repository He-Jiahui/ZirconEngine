mod state;
mod uniqueness;

use super::path::validate_runtime_plugin_package_root;

use self::state::new_runtime_plugin_package_root_array_state;

pub(super) fn validate_runtime_plugin_package_root_array(
    field_name: &str,
    roots: &[String],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_root_array_state();
    for root in roots {
        uniqueness::validate_runtime_plugin_package_root_uniqueness(
            field_name,
            root,
            &mut seen,
            diagnostics,
        );
        validate_runtime_plugin_package_root(field_name, root, diagnostics);
    }
}
