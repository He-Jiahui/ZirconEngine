use zircon_runtime_interface::{ZrByteBufferRef, ZrStatusCode};

use crate::core::framework::bridge::InterfaceSlot;
use crate::plugin::native::NativeHostBridgeCallScope;
use crate::plugin::{
    PluginModuleKind, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
    RuntimePluginBridgeLifecycleState,
};
use crate::scene::ecs::{SystemRef, SystemStage};

use super::super::behavior_validation::ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3;
use super::super::host_api_adapter::NativeDynamicAccess;
use super::super::registration_manifest::{
    NativePluginRegistrationManifest, NativePluginRegistrationManifestError,
    NativePluginRegistrationSystem,
};
use super::keys::live_key;
use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
use super::reports::{
    NativePluginRuntimeRegistrationReplayReport, NativePluginRuntimeRegistrationSystemReplay,
};
use super::runtime_behavior::runtime_plugins;
use super::NativePluginLiveHost;

pub(super) type NativePluginRegistrationReplayResult<T> =
    std::result::Result<T, NativePluginRegistrationReplayError>;

#[derive(Debug)]
pub(super) enum NativePluginRegistrationReplayError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimePluginNotLoaded {
        plugin_id: String,
    },
    UnsupportedManifestSchema {
        plugin_id: String,
        actual: String,
        expected: &'static str,
    },
    MissingRegistrationManifest {
        plugin_id: String,
    },
    InvalidRegistrationManifest {
        plugin_id: String,
        source: NativePluginRegistrationManifestError,
    },
    InvalidRegistrationSystem {
        plugin_id: String,
        system_id: String,
        source: NativePluginRegistrationManifestError,
    },
    BridgeMethodSlot {
        plugin_id: String,
        system_id: String,
        bridge_interface: String,
        bridge_method: String,
        source: String,
    },
    UnknownBridgeInterface {
        plugin_id: String,
        system_id: String,
        bridge_interface: String,
    },
    BridgeCallScope {
        plugin_id: String,
        source: String,
    },
    RegistryInternPluginModule {
        plugin_id: String,
        system_id: String,
        module: String,
        source: RuntimeExtensionRegistryError,
    },
    RegistryInternSystemSet {
        plugin_id: String,
        system_id: String,
        set_name: String,
        source: RuntimeExtensionRegistryError,
    },
    RegisterNativeSystem {
        plugin_id: String,
        system_id: String,
        source: RuntimeExtensionRegistryError,
    },
}

impl std::fmt::Display for NativePluginRegistrationReplayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLock(source) => write!(formatter, "{source}"),
            Self::RuntimePluginNotLoaded { plugin_id } => write!(
                formatter,
                "plugin {plugin_id} is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
            ),
            Self::UnsupportedManifestSchema {
                plugin_id,
                actual,
                expected,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration manifest schema `{actual}` is unsupported; expected {expected}"
            ),
            Self::MissingRegistrationManifest { plugin_id } => write!(
                formatter,
                "runtime plugin {plugin_id} has no registration manifest to replay"
            ),
            Self::InvalidRegistrationManifest { plugin_id, source } => {
                write!(formatter, "runtime plugin {plugin_id} {source}")
            }
            Self::InvalidRegistrationSystem {
                plugin_id,
                system_id,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` {source}"
            ),
            Self::BridgeMethodSlot {
                plugin_id,
                system_id,
                bridge_interface,
                bridge_method,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` failed to resolve bridge method `{bridge_interface}.{bridge_method}`: {source}"
            ),
            Self::UnknownBridgeInterface {
                plugin_id,
                system_id,
                bridge_interface,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` references unknown bridge interface `{bridge_interface}`"
            ),
            Self::BridgeCallScope { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to build native registration replay bridge call scope: {source}"
            ),
            Self::RegistryInternPluginModule {
                plugin_id,
                system_id,
                module,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to intern native registration manifest system `{system_id}` module `{module}`: {source}"
            ),
            Self::RegistryInternSystemSet {
                plugin_id,
                system_id,
                set_name,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to intern native registration manifest system `{system_id}` set `{set_name}`: {source}"
            ),
            Self::RegisterNativeSystem {
                plugin_id,
                system_id,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to register native registration manifest system `{system_id}`: {source}"
            ),
        }
    }
}

impl std::error::Error for NativePluginRegistrationReplayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LiveHostLock(source) => Some(source),
            Self::InvalidRegistrationManifest { source, .. }
            | Self::InvalidRegistrationSystem { source, .. } => Some(source),
            Self::RegistryInternPluginModule { source, .. }
            | Self::RegistryInternSystemSet { source, .. }
            | Self::RegisterNativeSystem { source, .. } => Some(source),
            Self::RuntimePluginNotLoaded { .. }
            | Self::UnsupportedManifestSchema { .. }
            | Self::MissingRegistrationManifest { .. }
            | Self::BridgeMethodSlot { .. }
            | Self::UnknownBridgeInterface { .. }
            | Self::BridgeCallScope { .. } => None,
        }
    }
}

impl NativePluginLiveHost {
    pub fn replay_runtime_registration_manifests_via_bridge(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginRuntimeRegistrationReplayReport, String> {
        let plugin_ids = {
            let loaded = match lock_loaded_native_plugins(&self.loaded) {
                Ok(loaded) => loaded,
                Err(error) => return Err(error.to_string()),
            };
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
        self.replay_runtime_plugin_registration_manifest_via_bridge_result(
            registry, lifecycle, plugin_id,
        )
        .map_err(|error| error.to_string())
    }

    pub(super) fn replay_runtime_plugin_registration_manifest_via_bridge_result(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        plugin_id: impl AsRef<str>,
    ) -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationReplayReport> {
        let plugin_id = plugin_id.as_ref();
        let source = self.runtime_registration_manifest_source(plugin_id)?;
        let manifest =
            NativePluginRegistrationManifest::from_toml(&source.text).map_err(|source| {
                NativePluginRegistrationReplayError::InvalidRegistrationManifest {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?;
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
    ) -> NativePluginRegistrationReplayResult<RuntimeRegistrationManifestSource> {
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginRegistrationReplayError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(
                || NativePluginRegistrationReplayError::RuntimePluginNotLoaded {
                    plugin_id: plugin_id.to_string(),
                },
            )?;
        let schema = plugin.runtime_registration_manifest_schema();
        if let Some(schema) = schema.map(str::trim).filter(|schema| !schema.is_empty()) {
            if schema != ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3 {
                return Err(
                    NativePluginRegistrationReplayError::UnsupportedManifestSchema {
                        plugin_id: plugin_id.to_string(),
                        actual: schema.to_string(),
                        expected: ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3,
                    },
                );
            }
        }
        let text = plugin
            .runtime_registration_manifest()
            .ok_or_else(
                || NativePluginRegistrationReplayError::MissingRegistrationManifest {
                    plugin_id: plugin_id.to_string(),
                },
            )?
            .to_string();
        Ok(RuntimeRegistrationManifestSource { text })
    }

    fn replay_runtime_registration_system(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        plugin_id: &str,
        system: &NativePluginRegistrationSystem,
    ) -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationSystemReplay> {
        let stage = system.stage().map_err(|source| {
            NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                source,
            }
        })?;
        let bridge_interface = system
            .bridge_interface()
            .map_err(
                |source| NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    source,
                },
            )?
            .to_string();
        let bridge_method = system
            .bridge_method()
            .map_err(
                |source| NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    source,
                },
            )?
            .to_string();
        let bridge_method_slot = self
            .runtime_bridge_method_slot(plugin_id, &bridge_interface, &bridge_method)
            .map_err(
                |source| NativePluginRegistrationReplayError::BridgeMethodSlot {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    bridge_interface: bridge_interface.clone(),
                    bridge_method: bridge_method.clone(),
                    source,
                },
            )?;
        let bridge_interface_slot = lifecycle
            .bridge_table()
            .resolve_slot(&bridge_interface)
            .ok_or_else(
                || NativePluginRegistrationReplayError::UnknownBridgeInterface {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    bridge_interface: bridge_interface.clone(),
                },
            )?;
        let bridge_call_scope = self
            .runtime_bridge_call_scope_from_installed_bindings(plugin_id, lifecycle)
            .map_err(
                |source| NativePluginRegistrationReplayError::BridgeCallScope {
                    plugin_id: plugin_id.to_string(),
                    source,
                },
            )?;
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
) -> NativePluginRegistrationReplayResult<()> {
    let owner = registry
        .intern_plugin_module(runtime_module_name(plugin_id, &system.module))
        .map_err(
            |source| NativePluginRegistrationReplayError::RegistryInternPluginModule {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                module: system.module.clone(),
                source,
            },
        )?;
    let sets = system
        .sets
        .iter()
        .map(|set_name| {
            registry
                .intern_system_set(set_name.clone())
                .map_err(
                    |source| NativePluginRegistrationReplayError::RegistryInternSystemSet {
                        plugin_id: plugin_id.to_string(),
                        system_id: system.id.clone(),
                        set_name: set_name.clone(),
                        source,
                    },
                )
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
    builder.register().map_err(
        |source| NativePluginRegistrationReplayError::RegisterNativeSystem {
            plugin_id: plugin_id.to_string(),
            system_id,
            source,
        },
    )
}

fn runtime_module_name(plugin_id: &str, module: &str) -> String {
    if module.contains('.') {
        module.to_string()
    } else {
        format!("{plugin_id}.{module}")
    }
}
