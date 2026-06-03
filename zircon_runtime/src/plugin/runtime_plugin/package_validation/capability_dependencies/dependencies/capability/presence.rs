use crate::plugin::PluginDependencyManifest;

pub(super) fn validate_runtime_plugin_package_dependency_capability_presence<'a>(
    dependency: &'a PluginDependencyManifest,
    diagnostics: &mut Vec<String>,
) -> Option<&'a str> {
    let Some(capability) = dependency.capability.as_deref() else {
        diagnostics.push(format!(
            "runtime plugin package manifest dependency `{}` must declare a capability",
            dependency.id
        ));
        return None;
    };
    Some(capability)
}
