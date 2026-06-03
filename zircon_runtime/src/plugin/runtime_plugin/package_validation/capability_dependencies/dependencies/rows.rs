mod pairs;

use crate::plugin::PluginPackageManifest;

use self::pairs::new_runtime_plugin_package_dependency_pair_state;
use super::row::validate_runtime_plugin_package_dependency_row;

pub(super) fn validate_runtime_plugin_package_dependency_rows(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_dependency_pair_state();
    for dependency in &package_manifest.dependencies {
        validate_runtime_plugin_package_dependency_row(dependency, &mut seen, diagnostics);
    }
}
