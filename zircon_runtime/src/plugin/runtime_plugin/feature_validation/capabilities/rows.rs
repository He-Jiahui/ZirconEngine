mod state;

use self::state::new_runtime_plugin_feature_capability_row_state;
use super::row::validate_runtime_plugin_feature_capability_row;

pub(super) fn validate_runtime_plugin_feature_capability_rows(
    capabilities: &[String],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_feature_capability_row_state();
    for capability in capabilities {
        validate_runtime_plugin_feature_capability_row(capability, &mut seen, diagnostics);
    }
}
