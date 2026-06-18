use std::collections::HashSet;

use crate::builtin::RuntimeTargetMode;

use super::super::RuntimePluginRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn base_capabilities_for_target(
    registrations: &[RuntimePluginRegistrationReport],
    enabled_plugins: &HashSet<String>,
    target: RuntimeTargetMode,
) -> HashSet<String> {
    let mut capabilities = HashSet::new();
    for registration in registrations {
        if !enabled_plugins.contains(&registration.package_manifest.id) {
            continue;
        }
        for module in &registration.package_manifest.modules {
            if module.target_modes.is_empty() || module.target_modes.contains(&target) {
                capabilities.extend(module.capabilities.iter().cloned());
            }
        }
    }
    capabilities
}
