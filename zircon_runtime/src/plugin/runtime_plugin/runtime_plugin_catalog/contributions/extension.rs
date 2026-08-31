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
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use crate::core::framework::bridge::{BridgeError, PluginInterface};
    use crate::core::ModuleDescriptor;
    use crate::plugin::RuntimeExtensionRegistry;

    use super::{
        merge_extension_registry_contributions,
        merge_extension_registry_contributions_for_runtime_modules,
    };

    trait MergeTestBridge: Send + Sync {
        fn sample(&self) -> i32;
    }

    impl PluginInterface for dyn MergeTestBridge {
        const INTERFACE_ID: &'static str = "test.final.merge.bridge.v1";
    }

    struct MergeTestProvider(i32);

    impl MergeTestBridge for MergeTestProvider {
        fn sample(&self) -> i32 {
            self.0
        }
    }

    #[test]
    fn target_filtered_merge_excludes_unselected_module_owned_interfaces() {
        let mut source = RuntimeExtensionRegistry::default();
        source
            .register_module(ModuleDescriptor::new("client.runtime", "Client"))
            .unwrap();
        source
            .register_module(ModuleDescriptor::new("server.runtime", "Server"))
            .unwrap();
        let server_owner = source.intern_plugin_module("server.runtime").unwrap();
        source
            .export_interface::<dyn MergeTestBridge>(server_owner, Arc::new(MergeTestProvider(7)))
            .unwrap();

        let mut merged = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        let mut fatal_diagnostics = Vec::new();
        merge_extension_registry_contributions_for_runtime_modules(
            &source,
            &HashSet::from(["client.runtime"]),
            &mut merged,
            &mut diagnostics,
            &mut fatal_diagnostics,
        );
        merged.finalize();

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");
        assert_eq!(merged.modules()[0].name, "client.runtime");
        assert!(merged
            .frozen_bridge_table()
            .resolve_slot(<dyn MergeTestBridge as PluginInterface>::INTERFACE_ID)
            .is_none());
    }

    #[test]
    fn interface_import_binds_to_final_merged_table_and_tracks_lifecycle() {
        let mut consumer = RuntimeExtensionRegistry::default();
        let consumer_owner = consumer.intern_plugin_module("consumer.runtime").unwrap();
        let imported = consumer
            .import_interface::<dyn MergeTestBridge>(consumer_owner)
            .unwrap();
        assert_eq!(
            imported.call(MergeTestBridge::sample),
            Err(BridgeError::Absent)
        );

        let mut provider = RuntimeExtensionRegistry::default();
        let provider_owner = provider.intern_plugin_module("provider.runtime").unwrap();
        provider
            .export_interface::<dyn MergeTestBridge>(provider_owner, Arc::new(MergeTestProvider(7)))
            .unwrap();

        let mut merged = RuntimeExtensionRegistry::default();
        let merged_provider_owner = merged.intern_plugin_module("provider.runtime").unwrap();
        let mut diagnostics = Vec::new();
        let mut fatal_diagnostics = Vec::new();
        merge_extension_registry_contributions(
            &consumer,
            &mut merged,
            &mut diagnostics,
            &mut fatal_diagnostics,
        );
        merge_extension_registry_contributions(
            &provider,
            &mut merged,
            &mut diagnostics,
            &mut fatal_diagnostics,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");

        merged.finalize();
        let table = merged.frozen_bridge_table();
        assert_eq!(imported.call(MergeTestBridge::sample), Ok(7));

        table.set_owner_enabled(merged_provider_owner, false);
        assert_eq!(
            imported.call(MergeTestBridge::sample),
            Err(BridgeError::NotEnabled)
        );

        let slot = table
            .resolve_slot(<dyn MergeTestBridge as PluginInterface>::INTERFACE_ID)
            .unwrap();
        table
            .reload_provider::<dyn MergeTestBridge>(slot, Arc::new(MergeTestProvider(11)))
            .unwrap();
        table.set_owner_enabled(merged_provider_owner, true);
        assert_eq!(imported.call(MergeTestBridge::sample), Ok(11));
        assert_eq!(table.diagnostics(slot).unwrap().not_enabled_calls, 1);

        merged.revoke_owner_registrations(merged_provider_owner);
        let current_table = merged.frozen_bridge_table();
        assert!(current_table
            .resolve_slot(<dyn MergeTestBridge as PluginInterface>::INTERFACE_ID)
            .is_none());
        table
            .reload_provider::<dyn MergeTestBridge>(slot, Arc::new(MergeTestProvider(13)))
            .unwrap();
        table.set_owner_enabled(merged_provider_owner, true);
        assert_eq!(
            imported.call(MergeTestBridge::sample),
            Err(BridgeError::Absent),
            "surviving imports must be rebound away from the revoked table"
        );
    }
}
