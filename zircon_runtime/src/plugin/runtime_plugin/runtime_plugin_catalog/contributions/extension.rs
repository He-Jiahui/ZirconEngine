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
    for (owner, import) in extensions.plugin_interface_imports() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_interface_import(target_owner, import.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    #[cfg(feature = "graphics")]
    for (owner, descriptor) in extensions.geometry_source_entries() {
        let result = intern_target_owner(registry, extensions, owner).and_then(|target_owner| {
            registry.register_geometry_source_for_owner(target_owner, descriptor.clone())
        });
        push_runtime_extension_result(result, diagnostics, fatal_diagnostics);
    }
    #[cfg(feature = "graphics")]
    for (owner, descriptor) in extensions.shading_model_entries() {
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
    use std::sync::Arc;

    use crate::core::framework::bridge::{BridgeError, PluginInterface};
    use crate::plugin::RuntimeExtensionRegistry;

    use super::merge_extension_registry_contributions;

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
