use std::collections::HashSet;

use crate::plugin::{PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError};

use super::super::descriptor_contributions::merge_descriptor_extension_registry_contributions;
#[cfg(feature = "graphics")]
use super::super::render_contributions::merge_render_extension_registry_contributions;
use super::diagnostic::push_runtime_extension_result;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_extension_registry_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    merge_extension_registry_contributions_with_module_filter(
        extensions,
        None,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_extension_registry_contributions_for_runtime_modules(
    extensions: &RuntimeExtensionRegistry,
    selected_runtime_module_names: &HashSet<&str>,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    merge_extension_registry_contributions_with_module_filter(
        extensions,
        Some(selected_runtime_module_names),
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

// 将注册报告的贡献投影到目录独立持有的注册表，使接口与系统等 owner 归属采用同一目录编号空间。
// 各源表的 owner ID 只在本表有效，进入目录后需按模块名重新驻留。
fn merge_extension_registry_contributions_with_module_filter(
    extensions: &RuntimeExtensionRegistry,
    selected_runtime_module_names: Option<&HashSet<&str>>,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for module in extensions.modules() {
        if selected_runtime_module_names
            .is_some_and(|module_names| !module_names.contains(module.name.as_str()))
        {
            continue;
        }
        push_runtime_extension_result(
            registry.register_module(module.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for (owner, resource) in extensions.plugin_resources() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_resource_registration(target_owner, resource.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, event) in extensions.plugin_events() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_event_registration(target_owner, event.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, system) in extensions.plugin_systems() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_system_registration(target_owner, system.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, system) in extensions.plugin_runtime_systems() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_runtime_scene_system_registration(target_owner, system.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, export) in extensions.plugin_interfaces() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_interface_export(target_owner, export.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    for (owner, import) in extensions.plugin_interface_imports() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_interface_import(target_owner, import.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    #[cfg(feature = "graphics")]
    for (owner, descriptor) in extensions.geometry_source_entries() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_geometry_source_for_owner(target_owner, descriptor.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    #[cfg(feature = "graphics")]
    for (owner, descriptor) in extensions.shading_model_entries() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_shading_model_for_owner(target_owner, descriptor.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    #[cfg(feature = "graphics")]
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
    // Project callbacks and owner listeners use the same owner remap as every other
    // contribution. Linked plugin reports enter here before RuntimePreparedProject consumes the
    // merged report; direct registry fixtures do not cover this production path.
    for (owner, serializer) in extensions.scene_component_codecs() {
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_scene_component_codec_for_owner(target_owner, serializer)
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    let mut projected_listener_owners = HashSet::new();
    for owner in extensions.owner_revocation_listener_owners() {
        if !projected_listener_owners.insert(owner) {
            continue;
        }
        if !owner_is_selected(extensions, owner, selected_runtime_module_names) {
            continue;
        }
        match intern_target_owner(registry, extensions, owner) {
            Ok(target_owner) => {
                extensions.project_owner_revocation_listeners_to(owner, registry, target_owner);
            }
            Err(error) => {
                push_runtime_extension_result(Err(error), diagnostics, fatal_diagnostics);
            }
        }
    }
}

fn owner_is_selected(
    extensions: &RuntimeExtensionRegistry,
    owner: PluginModuleId,
    selected_runtime_module_names: Option<&HashSet<&str>>,
) -> bool {
    selected_runtime_module_names.is_none_or(|module_names| {
        extensions
            .plugin_module_name(owner)
            .is_some_and(|module_name| module_names.contains(module_name))
    })
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

#[cfg(test)]
#[path = "tests/extension.rs"]
mod tests;

#[cfg(test)]
#[path = "normal_catalog_consumer/tests/mod.rs"]
mod normal_catalog_consumer;
