use crate::plugin::PluginDependencyManifest;

pub(super) fn validate_runtime_plugin_package_dependency_capability_presence<'a>(
    dependency: &'a PluginDependencyManifest,
    _diagnostics: &mut Vec<String>,
) -> Option<&'a str> {
    dependency.capability.as_deref()
}
