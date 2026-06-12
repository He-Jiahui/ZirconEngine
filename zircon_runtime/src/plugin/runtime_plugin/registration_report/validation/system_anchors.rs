use crate::plugin::{PluginModuleKind, PluginPackageManifest, RuntimeExtensionRegistry};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_system_anchors(
    package_manifest: &PluginPackageManifest,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for module in package_manifest
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
    {
        for anchor in &module.system_anchors {
            if !runtime_module_registered_system(extensions, &module.name, anchor) {
                diagnostics.push(format!(
                    "runtime plugin module `{}` declares system anchor `{anchor}` but did not register a matching runtime system",
                    module.name
                ));
            }
        }
    }
}

fn runtime_module_registered_system(
    extensions: &RuntimeExtensionRegistry,
    module_name: &str,
    system_id: &str,
) -> bool {
    extensions.plugin_systems().any(|(owner, system)| {
        system.id == system_id && extensions.plugin_module_name(owner) == Some(module_name)
    }) || extensions.plugin_runtime_systems().any(|(owner, system)| {
        system.id == system_id && extensions.plugin_module_name(owner) == Some(module_name)
    })
}
