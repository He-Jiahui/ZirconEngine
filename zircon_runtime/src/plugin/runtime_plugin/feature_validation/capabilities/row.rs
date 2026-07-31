mod field;
mod namespace;
mod uniqueness;

use self::{
    field::validate_runtime_plugin_feature_capability_field,
    namespace::validate_runtime_plugin_feature_capability_namespace,
    uniqueness::validate_runtime_plugin_feature_capability_row_uniqueness,
};

pub(super) fn validate_runtime_plugin_feature_capability_row(
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_capability_field(capability, diagnostics);
    validate_runtime_plugin_feature_capability_namespace(capability, diagnostics);
    validate_runtime_plugin_feature_capability_row_uniqueness(
        capability,
        is_duplicate,
        diagnostics,
    );
}
