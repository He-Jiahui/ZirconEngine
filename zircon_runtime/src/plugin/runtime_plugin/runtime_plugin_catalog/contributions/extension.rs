use crate::plugin::{PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError};

use super::super::descriptor_contributions::merge_descriptor_extension_registry_contributions;
use super::super::render_contributions::merge_render_extension_registry_contributions;
use super::diagnostic::push_runtime_extension_result;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_extension_registry_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for module in extensions.modules() {
        push_runtime_extension_result(
            registry.register_module(module.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for (owner, resource) in extensions.plugin_resources() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_resource_registration(target_owner, resource.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, event) in extensions.plugin_events() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_event_registration(target_owner, event.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, system) in extensions.plugin_systems() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_system_registration(target_owner, system.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, system) in extensions.plugin_runtime_systems() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_runtime_scene_system_registration(target_owner, system.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, export) in extensions.plugin_interfaces() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_interface_export(target_owner, export.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    merge_render_extension_registry_contributions(
        extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
    merge_descriptor_extension_registry_contributions(
        extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

fn intern_target_owner(
    target: &mut RuntimeExtensionRegistry,
    source: &RuntimeExtensionRegistry,
    owner: PluginModuleId,
) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
    let Some(module_name) = source.plugin_module_name(owner) else {
        return Err(RuntimeExtensionRegistryError::InvalidPluginModule(format!(
            "unknown plugin module owner {}",
            owner.raw()
        )));
    };
    target.intern_plugin_module(module_name.to_string())
}
