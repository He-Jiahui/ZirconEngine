mod state;

use self::state::new_runtime_plugin_package_capability_status_bevy_reference_row_state;
use super::row::validate_runtime_plugin_package_capability_status_bevy_reference_row;

pub(super) fn validate_runtime_plugin_package_capability_status_bevy_reference_rows(
    capability: &str,
    bevy_references: &[String],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_capability_status_bevy_reference_row_state();
    for reference in bevy_references {
        validate_runtime_plugin_package_capability_status_bevy_reference_row(
            capability,
            reference,
            &mut seen,
            diagnostics,
        );
    }
}
