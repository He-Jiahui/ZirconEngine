mod namespace;
mod uniqueness;

use self::namespace::validate_runtime_plugin_package_capability_namespace;
use self::uniqueness::validate_runtime_plugin_package_capability_row_uniqueness;

pub(super) fn validate_runtime_plugin_package_capability_row<'a>(
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_namespace(capability, diagnostics);
    validate_runtime_plugin_package_capability_row_uniqueness(capability, seen, diagnostics);
}
