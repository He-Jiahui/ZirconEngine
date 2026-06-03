mod state;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_option_duplicate_row_state;
use self::uniqueness::validate_runtime_plugin_package_option_key_uniqueness;

pub(super) fn validate_duplicate_plugin_options(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_option_duplicate_row_state();
    for option in &package_manifest.options {
        validate_runtime_plugin_package_option_key_uniqueness(
            option.key.as_str(),
            &mut seen,
            diagnostics,
        );
    }
}
