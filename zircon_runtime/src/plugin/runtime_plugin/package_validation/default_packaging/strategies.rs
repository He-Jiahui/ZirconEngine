mod state;
mod uniqueness;

use crate::core::framework::project::ExportPackagingStrategy;

use self::state::new_runtime_plugin_default_packaging_strategy_state;
use self::uniqueness::validate_runtime_plugin_default_packaging_strategy_uniqueness;

pub(super) fn validate_runtime_plugin_default_packaging_strategies(
    owner: &str,
    default_packaging: &[ExportPackagingStrategy],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_default_packaging_strategy_state();
    for strategy in default_packaging.iter().copied() {
        validate_runtime_plugin_default_packaging_strategy_uniqueness(
            owner,
            strategy,
            &mut seen,
            diagnostics,
        );
    }
}
