use std::collections::HashSet;

use super::super::super::package_validation::RuntimePluginPackageValidationProjection;
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_system_anchors(
    _package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    let registered_systems = extensions
        .plugin_systems()
        .filter_map(|(owner, system)| {
            extensions
                .plugin_module_name(owner)
                .map(|module_name| (module_name, system.id.as_str()))
        })
        .chain(
            extensions
                .plugin_runtime_systems()
                .filter_map(|(owner, system)| {
                    extensions
                        .plugin_module_name(owner)
                        .map(|module_name| (module_name, system.id.as_str()))
                }),
        )
        .collect::<HashSet<_>>();

    for (module_name, anchor) in projection.runtime_system_anchors() {
        if !registered_systems.contains(&(module_name, anchor)) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` declares system anchor `{anchor}` but did not register a matching runtime system"
            ));
        }
    }
}
