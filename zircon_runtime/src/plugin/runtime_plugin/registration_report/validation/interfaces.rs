use crate::plugin::{PluginModuleKind, PluginPackageManifest, RuntimeExtensionRegistry};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_interfaces(
    package_manifest: &PluginPackageManifest,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    let runtime_modules = package_manifest
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.name.as_str())
        .collect::<Vec<_>>();

    for declared in &package_manifest.provides_interfaces {
        if !package_exported_interface(extensions, &runtime_modules, &declared.id) {
            diagnostics.push(format!(
                "runtime plugin package `{}` declares interface `{}` but no runtime module exported it",
                package_manifest.id, declared.id
            ));
        }
    }

    for (owner, export) in extensions.plugin_interfaces() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !package_manifest
            .provides_interfaces
            .iter()
            .any(|declared| declared.id == export.interface_id())
        {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` exported interface `{}` but package manifest did not declare it",
                export.interface_id()
            ));
        }
    }
}

fn package_exported_interface(
    extensions: &RuntimeExtensionRegistry,
    runtime_modules: &[&str],
    interface_id: &str,
) -> bool {
    extensions.plugin_interfaces().any(|(owner, export)| {
        export.interface_id() == interface_id
            && extensions
                .plugin_module_name(owner)
                .is_some_and(|module_name| runtime_modules.contains(&module_name))
    })
}
