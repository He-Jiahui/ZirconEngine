use std::collections::HashSet;

use super::super::super::package_validation::RuntimePluginPackageValidationProjection;
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

// 双向核对清单声明与实际扩展注册；只把所属运行时模块的接口计入包级满足关系。
pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_interfaces(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    let exported_interface_rows = extensions.plugin_interfaces();
    let (exported_interface_capacity, _) = exported_interface_rows.size_hint();
    let mut exported_interfaces = HashSet::with_capacity(exported_interface_capacity);
    for (owner, export) in exported_interface_rows {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if projection.is_runtime_module(module_name) {
            exported_interfaces.insert(export.interface_id());
        }
    }

    let imported_interface_rows = extensions.plugin_interface_imports();
    let (imported_interface_capacity, _) = imported_interface_rows.size_hint();
    let mut imported_interfaces = HashSet::with_capacity(imported_interface_capacity);
    for (owner, import) in imported_interface_rows {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if projection.is_runtime_module(module_name) {
            imported_interfaces.insert(import.interface_id());
        }
    }

    for interface_id in projection.provided_interface_ids() {
        if !exported_interfaces.contains(interface_id) {
            diagnostics.push(format!(
                "runtime plugin package `{}` declares interface `{}` but no runtime module exported it",
                package_manifest.id, interface_id
            ));
        }
    }

    for (owner, export) in extensions.plugin_interfaces() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !projection.declares_provided_interface(export.interface_id()) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` exported interface `{}` but package manifest did not declare it",
                export.interface_id()
            ));
        }
    }

    for interface_id in projection.dependency_interface_ids() {
        if !imported_interfaces.contains(interface_id) {
            diagnostics.push(format!(
                "runtime plugin package `{}` declares dependency interface `{interface_id}` but no runtime module imported it",
                package_manifest.id
            ));
        }
    }

    for (owner, import) in extensions.plugin_interface_imports() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !projection.declares_dependency_interface(import.interface_id()) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` imported interface `{}` but package dependencies did not declare it",
                import.interface_id()
            ));
        }
    }
}

#[cfg(test)]
#[path = "tests/interfaces.rs"]
mod tests;
