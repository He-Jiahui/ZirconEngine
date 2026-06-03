use crate::plugin::RuntimeExtensionRegistry;

use super::super::extension_merge::merge_runtime_extensions;
use super::super::RuntimePluginRegistrationReport;
use super::RuntimeExtensionCatalogReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn runtime_extension_report(
    registrations: &[RuntimePluginRegistrationReport],
) -> RuntimeExtensionCatalogReport {
    let mut registry = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for registration in registrations {
        merge_runtime_extensions(
            registration,
            &mut registry,
            &mut diagnostics,
            &mut fatal_diagnostics,
        );
    }
    RuntimeExtensionCatalogReport {
        registry,
        diagnostics,
        fatal_diagnostics,
    }
}
