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
    if dependency.capability.is_none() && dependency.interfaces.is_empty() {
        diagnostics.push(format!(
            "runtime plugin package manifest dependency `{}` must declare a capability or at least one interface",
            dependency.id
        ));
        return;
    }
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
