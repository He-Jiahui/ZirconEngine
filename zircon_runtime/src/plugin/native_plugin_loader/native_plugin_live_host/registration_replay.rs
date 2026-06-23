use zircon_runtime_interface::{ZrByteBufferRef, ZrStatusCode};

use crate::plugin::native::NativeHostBridgeCallScope;
use crate::plugin::{
    InterfaceSlot, PluginModuleKind, RuntimeExtensionRegistry, RuntimePluginBridgeLifecycleState,
};
use crate::scene::ecs::{SystemRef, SystemStage};

use super::super::behavior_validation::ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3;
use super::super::host_api_adapter::NativeDynamicAccess;
use super::super::registration_manifest::{
    NativePluginRegistrationManifest, NativePluginRegistrationSystem,
};
use super::diagnostics::unloaded_plugin_error;
use super::keys::live_key;
use super::loading::lock_loaded_native_plugins;
use super::reports::{
    NativePluginRuntimeRegistrationReplayReport, NativePluginRuntimeRegistrationSystemReplay,
};
use super::runtime_behavior::runtime_plugins;
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn replay_runtime_registration_manifests_via_bridge(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginRuntimeRegistrationReplayReport, String> {
        let plugin_ids = {
            let loaded = lock_loaded_native_plugins(&self.loaded)?;
            runtime_plugins(&loaded)
                .filter(|(_, plugin)| plugin.runtime_registration_manifest().is_some())
                .map(|(plugin_id, _)| plugin_id)
                .collect::<Vec<_>>()
        };

        let mut report = NativePluginRuntimeRegistrationReplayReport::default();
        for plugin_id in plugin_ids {
            match self.replay_runtime_plugin_registration_manifest_via_bridge(
                registry, lifecycle, &plugin_id,
            ) {
                Ok(mut plugin_report) => report.append(&mut plugin_report),
                Err(error) => {
                    report.skipped_plugin_ids.push(plugin_id);
                    report.diagnostics.push(error);
                }
            }
        }
        report.diagnostics.sort();
        report.diagnostics.dedup();
        report.skipped_plugin_ids.sort();
        report.skipped_plugin_ids.dedup();
        Ok(report)
    }

    pub fn replay_runtime_plugin_registration_manifest_via_bridge(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        plugin_id: impl AsRef<str>,
    ) -> Result<NativePluginRuntimeRegistrationReplayReport, String> {
        let plugin_id = plugin_id.as_ref();
        let source = self.runtime_registration_manifest_source(plugin_id)?;
        let manifest = NativePluginRegistrationManifest::from_toml(&source.text)
            .map_err(|error| format!("runtime plugin {plugin_id} {error}"))?;
        let mut report = NativePluginRuntimeRegistrationReplayReport::default();
        for system in &manifest.systems {
            let system_report =
                self.replay_runtime_registration_system(registry, lifecycle, plugin_id, system)?;
            report.registered_systems.push(system_report);
        }
        Ok(report)
    }

    fn runtime_registration_manifest_source(
        &self,
        plugin_id: &str,
    ) -> Result<RuntimeRegistrationManifestSource, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        let schema = plugin.runtime_registration_manifest_schema();
        if let Some(schema) = schema.map(str::trim).filter(|schema| !schema.is_empty()) {
            if schema != ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3 {
                return Err(format!(
                    "runtime plugin {plugin_id} registration manifest schema `{schema}` is unsupported; expected {ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3}"
                ));
            }
        }
        let text = plugin
            .runtime_registration_manifest()
            .ok_or_else(|| {
                format!("runtime plugin {plugin_id} has no registration manifest to replay")
            })?
            .to_string();
        Ok(RuntimeRegistrationManifestSource { text })
    }

    fn replay_runtime_registration_system(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        plugin_id: &str,
        system: &NativePluginRegistrationSystem,
    ) -> Result<NativePluginRuntimeRegistrationSystemReplay, String> {
        let stage = system.stage()?;
        let bridge_interface = system.bridge_interface()?.to_string();
        let bridge_method = system.bridge_method()?.to_string();
        let bridge_method_slot =
            self.runtime_bridge_method_slot(plugin_id, &bridge_interface, &bridge_method)?;
        let bridge_interface_slot = lifecycle
            .bridge_table()
            .resolve_slot(&bridge_interface)
            .ok_or_else(|| {
                format!(
                    "runtime plugin {plugin_id} registration system `{}` references unknown bridge interface `{bridge_interface}`",
                    system.id
                )
            })?;
        let bridge_call_scope =
            self.runtime_bridge_call_scope_from_installed_bindings(plugin_id, lifecycle)?;
        register_bridge_replay_system(
            registry,
            plugin_id,
            system,
            stage,
            bridge_interface_slot,
            bridge_method_slot,
            bridge_call_scope,
        )?;
        Ok(NativePluginRuntimeRegistrationSystemReplay {
            plugin_id: plugin_id.to_string(),
            module: system.module.clone(),
            system_id: system.id.clone(),
            stage,
            order: system.order,
            bridge_interface,
            bridge_method,
        })
    }
}

struct RuntimeRegistrationManifestSource {
    text: String,
}

fn register_bridge_replay_system(
    registry: &mut RuntimeExtensionRegistry,
    plugin_id: &str,
    system: &NativePluginRegistrationSystem,
    stage: SystemStage,
    bridge_interface_slot: InterfaceSlot,
    bridge_method_slot: u32,
    bridge_call_scope: NativeHostBridgeCallScope,
) -> Result<(), String> {
    let owner = registry
        .intern_plugin_module(runtime_module_name(plugin_id, &system.module))
        .map_err(|error| error.to_string())?;
    let sets = system
        .sets
        .iter()
        .map(|set_name| {
            registry
                .intern_system_set(set_name.clone())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let system_id = system.id.clone();
    let mut builder = registry
        .register_native_system::<NativeDynamicAccess, _>(owner, system.id.clone(), stage, {
            move |()| {
                let api = bridge_call_scope.api();
                let Some(call) = api.bridge.call else {
                    return;
                };
                let status = unsafe {
                    call(
                        bridge_call_scope.handle(),
                        bridge_interface_slot.raw(),
                        bridge_method_slot,
                        std::ptr::null(),
                        0,
                        ZrByteBufferRef::empty(),
                    )
                };
                let _ = status.status_code() == ZrStatusCode::Ok;
            }
        })
        .with_order(system.order);
    for set in sets {
        builder = builder.in_set(set);
    }
    for before in &system.before {
        builder = builder.before(SystemRef::System(before.clone()));
    }
    for after in &system.after {
        builder = builder.after(SystemRef::System(after.clone()));
    }
    builder.register().map_err(|error| {
        format!(
            "runtime plugin {plugin_id} failed to register native registration manifest system `{system_id}`: {error}"
        )
    })
}

fn runtime_module_name(plugin_id: &str, module: &str) -> String {
    if module.contains('.') {
        module.to_string()
    } else {
        format!("{plugin_id}.{module}")
    }
}
