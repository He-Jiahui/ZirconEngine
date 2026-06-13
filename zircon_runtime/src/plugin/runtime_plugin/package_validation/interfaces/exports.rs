use crate::plugin::PluginPackageManifest;

use super::super::{
    validate_runtime_plugin_package_namespace, validate_runtime_plugin_package_token,
};

pub(super) fn validate_runtime_plugin_package_provided_interfaces(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = Vec::new();
    for interface in &package_manifest.provides_interfaces {
        validate_runtime_plugin_package_namespace(
            "provided interface id",
            interface.id.as_str(),
            diagnostics,
        );
        if seen.contains(&interface.id.as_str()) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{}` must be unique",
                interface.id
            ));
        } else {
            seen.push(interface.id.as_str());
        }
        validate_interface_methods(&interface.id, interface, diagnostics);
    }
}

fn validate_interface_methods(
    interface_id: &str,
    interface: &crate::plugin::PluginInterfaceManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen_names = Vec::new();
    let mut seen_slots = Vec::new();
    for method in &interface.methods {
        validate_runtime_plugin_package_token(
            "provided interface method name",
            method.name.as_str(),
            diagnostics,
        );
        if seen_names.contains(&method.name.as_str()) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{interface_id}` method `{}` must be unique",
                method.name
            ));
        } else {
            seen_names.push(method.name.as_str());
        }
        if seen_slots.contains(&method.method_slot) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{interface_id}` method slot {} must be unique",
                method.method_slot
            ));
        } else {
            seen_slots.push(method.method_slot);
        }
        for parameter in &method.parameters {
            validate_runtime_plugin_package_token(
                "provided interface method parameter name",
                parameter.name.as_str(),
                diagnostics,
            );
        }
        let mut seen_capabilities = Vec::new();
        for capability in &method.required_capabilities {
            validate_runtime_plugin_package_namespace(
                "provided interface method required capability",
                capability,
                diagnostics,
            );
            if seen_capabilities.contains(&capability.as_str()) {
                diagnostics.push(format!(
                    "runtime plugin package manifest provided interface `{interface_id}` method `{}` required capability `{capability}` must be unique",
                    method.name
                ));
            } else {
                seen_capabilities.push(capability.as_str());
            }
        }
    }
}
