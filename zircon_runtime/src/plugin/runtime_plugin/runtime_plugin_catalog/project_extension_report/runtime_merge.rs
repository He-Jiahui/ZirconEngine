use std::collections::HashSet;

use crate::core::CoreError;
use crate::plugin::RuntimeExtensionRegistry;

use super::super::extension_merge::merge_runtime_extensions;
use super::super::registration::order::order_runtime_plugin_registration_report_refs;
use super::super::RuntimePluginRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_enabled_runtime_extensions(
    registrations: &[RuntimePluginRegistrationReport],
    enabled_plugin_ids: &HashSet<String>,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) -> Result<(), CoreError> {
    let registrations = registrations
        .iter()
        .filter(|registration| enabled_plugin_ids.contains(&registration.package_manifest.id))
        .collect::<Vec<_>>();
    for registration in order_runtime_plugin_registration_report_refs(registrations)? {
        merge_runtime_extensions(registration, registry, diagnostics, fatal_diagnostics);
    }
    Ok(())
}
