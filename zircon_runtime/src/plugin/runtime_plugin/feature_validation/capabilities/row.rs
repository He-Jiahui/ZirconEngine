mod field;
mod namespace;
mod uniqueness;

use self::{
    field::validate_runtime_plugin_feature_capability_field,
    namespace::validate_runtime_plugin_feature_capability_namespace,
    uniqueness::validate_runtime_plugin_feature_capability_row_uniqueness,
};

pub(super) fn validate_runtime_plugin_feature_capability_row<'a>(
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_capability_field(capability, diagnostics);
    validate_runtime_plugin_feature_capability_namespace(capability, diagnostics);
    validate_runtime_plugin_feature_capability_row_uniqueness(capability, seen, diagnostics);
}
