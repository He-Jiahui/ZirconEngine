mod namespace;
mod presence;

use crate::plugin::PluginDependencyManifest;

use self::{
    namespace::validate_runtime_plugin_package_dependency_capability_namespace,
    presence::validate_runtime_plugin_package_dependency_capability_presence,
};

pub(super) fn validate_runtime_plugin_package_dependency_capability<'a>(
    dependency: &'a PluginDependencyManifest,
    diagnostics: &mut Vec<String>,
) -> Option<&'a str> {
    let capability =
        validate_runtime_plugin_package_dependency_capability_presence(dependency, diagnostics)?;
    validate_runtime_plugin_package_dependency_capability_namespace(capability, diagnostics);
    Some(capability)
}
