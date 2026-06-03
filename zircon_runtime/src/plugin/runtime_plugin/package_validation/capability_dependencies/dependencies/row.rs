mod capability;
mod pair;
mod provider;

use crate::plugin::PluginDependencyManifest;

use self::capability::validate_runtime_plugin_package_dependency_row_capability;
use self::pair::validate_runtime_plugin_package_dependency_row_pair;
use self::provider::validate_runtime_plugin_package_dependency_provider;

pub(super) fn validate_runtime_plugin_package_dependency_row<'a>(
    dependency: &'a PluginDependencyManifest,
    seen: &mut Vec<(&'a str, &'a str)>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_dependency_provider(dependency, diagnostics);
    let Some(capability) =
        validate_runtime_plugin_package_dependency_row_capability(dependency, diagnostics)
    else {
        return;
    };
    validate_runtime_plugin_package_dependency_row_pair(
        dependency.id.as_str(),
        capability,
        seen,
        diagnostics,
    );
}
