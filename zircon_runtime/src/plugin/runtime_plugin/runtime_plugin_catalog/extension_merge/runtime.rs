use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::{
    merge_extension_registry_contributions,
    merge_extension_registry_contributions_for_runtime_modules,
};
use super::super::runtime_module_target::runtime_module_names_for_target;
use super::super::RuntimePluginRegistrationReport;
use super::diagnostic::push_fatal_diagnostic;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_extensions(
    registration: &RuntimePluginRegistrationReport,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    merge_runtime_extensions_with_module_filter(
        registration,
        None,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_extensions_for_target(
    registration: &RuntimePluginRegistrationReport,
    target: RuntimeTargetMode,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    let selected_module_names =
        runtime_module_names_for_target(&registration.package_manifest.modules, target)
            .collect::<HashSet<_>>();
    merge_runtime_extensions_with_module_filter(
        registration,
        Some(&selected_module_names),
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

fn merge_runtime_extensions_with_module_filter(
    registration: &RuntimePluginRegistrationReport,
    selected_module_names: Option<&HashSet<&str>>,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for diagnostic in &registration.diagnostics {
        push_fatal_diagnostic(
            diagnostics,
            fatal_diagnostics,
            format!(
                "runtime plugin {} diagnostic: {diagnostic}",
                registration.package_manifest.id
            ),
        );
    }
    let plugin_id = registration.package_manifest.id.clone();
    for manager in registration.extensions.managers() {
        if let Err(error) = registry.register_manager(plugin_id.clone(), manager.clone()) {
            push_fatal_diagnostic(diagnostics, fatal_diagnostics, error.to_string());
        }
    }
    if let Some(selected_module_names) = selected_module_names {
        merge_extension_registry_contributions_for_runtime_modules(
            &registration.extensions,
            selected_module_names,
            registry,
            diagnostics,
            fatal_diagnostics,
        );
    } else {
        merge_extension_registry_contributions(
            &registration.extensions,
            registry,
            diagnostics,
            fatal_diagnostics,
        );
    }
}
