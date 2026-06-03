use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::merge_extension_registry_contributions;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::diagnostic::push_fatal_diagnostic;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_feature_extensions(
    registration: &RuntimePluginFeatureRegistrationReport,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for diagnostic in &registration.diagnostics {
        push_fatal_diagnostic(
            diagnostics,
            fatal_diagnostics,
            format!(
                "runtime plugin feature {} diagnostic: {diagnostic}",
                registration.manifest.id
            ),
        );
    }
    for manager in registration.extensions.managers() {
        if let Err(error) =
            registry.register_manager(registration.manifest.id.clone(), manager.clone())
        {
            push_fatal_diagnostic(diagnostics, fatal_diagnostics, error.to_string());
        }
    }
    merge_extension_registry_contributions(
        &registration.extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}
