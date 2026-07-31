mod field;
mod path;
mod uniqueness;

use self::{
    field::validate_runtime_plugin_package_capability_status_bevy_reference_row_field,
    path::validate_runtime_plugin_package_capability_status_bevy_reference_row_path,
    uniqueness::validate_runtime_plugin_package_capability_status_bevy_reference_row_uniqueness,
};

pub(super) fn validate_runtime_plugin_package_capability_status_bevy_reference_row(
    capability: &str,
    reference: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_bevy_reference_row_field(
        reference,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_bevy_reference_row_path(
        capability,
        reference,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_bevy_reference_row_uniqueness(
        capability,
        reference,
        is_duplicate,
        diagnostics,
    );
}
