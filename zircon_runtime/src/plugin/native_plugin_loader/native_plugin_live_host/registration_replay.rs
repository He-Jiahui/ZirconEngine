use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime_interface::{ZrByteBufferRef, ZrStatusCode};

use crate::core::framework::bridge::InterfaceSlot;
use crate::plugin::native::NativeHostBridgeCallScope;
use crate::plugin::{
    FrozenBridgeTable, PluginModuleKind, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
    RuntimePluginBridgeLifecycleState,
};
use crate::scene::ecs::{SystemRef, SystemStage};

use super::super::behavior_validation::ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3;
use super::super::host_callbacks::granted_capabilities_for_entry;
use super::super::registration_manifest::{
    NativePluginRegistrationManifest, NativePluginRegistrationManifestError,
    NativePluginRegistrationSystem, NativeSystemAccessAuthority, NativeSystemAccessAuthorityError,
    NativeSystemAccessPlan,
};
use super::super::LoadedNativePlugin;
use super::bridge_methods::{NativePluginBridgeMethodError, NativePluginBridgeMethodResult};
use super::keys::live_key;
use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
use super::reports::{
    NativePluginRuntimeRegistrationReplayReport, NativePluginRuntimeRegistrationSystemReplay,
};
use super::runtime_behavior::runtime_plugins;
use super::NativePluginLiveHost;

pub(super) type NativePluginRegistrationReplayResult<T> =
    std::result::Result<T, NativePluginRegistrationReplayError>;

/// One frozen bridge generation is built for a plugin replay. Method lookup is used only while
/// registering systems; every registered callback retains the shared call scope instead of a
/// private manifest/binding/method-map rebuild.
pub(super) struct NativePluginRegistrationReplayBridgeContext {
    pub(super) revision: u64,
    pub(super) bridge_table: FrozenBridgeTable,
    // System registration resolves names once into these slot tables. The registered callback
    // keeps only u32 slots, so no string lookup reaches the runtime bridge call path.
    pub(super) method_slots: Arc<HashMap<String, HashMap<String, u32>>>,
    pub(super) bridge_call_scope: Arc<NativeHostBridgeCallScope>,
}

impl std::fmt::Debug for NativePluginRegistrationReplayBridgeContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginRegistrationReplayBridgeContext")
            .field("revision", &self.revision)
            .field("interfaces", &self.method_slots.len())
            .field("methods", &self.bridge_call_scope.method_count())
            .finish()
    }
}

impl NativePluginRegistrationReplayBridgeContext {
    pub(super) fn method_slot_result(
        &self,
        plugin_id: &str,
        interface_id: &str,
        method_name: &str,
    ) -> NativePluginBridgeMethodResult<u32> {
        self.method_slots
            .get(interface_id)
            .and_then(|methods| methods.get(method_name))
            .copied()
            .ok_or_else(
                || NativePluginBridgeMethodError::MissingDeclaredBridgeMethod {
                    plugin_id: plugin_id.to_string(),
                    interface_id: interface_id.to_string(),
                    method_name: method_name.to_string(),
                },
            )
    }

    fn bridge_call_scope(&self) -> Arc<NativeHostBridgeCallScope> {
        self.bridge_call_scope.clone()
    }

    fn matches(&self, revision: u64, lifecycle: &RuntimePluginBridgeLifecycleState) -> bool {
        self.revision == revision
            && self
                .bridge_table
                .shares_storage_with(lifecycle.bridge_table())
    }
}

/// Immutable replay inputs for one runtime plugin load/binding/bridge-table generation.
///
/// The cache stores this behind an `Arc`: systems registered from an older generation retain its
/// call scope while a later binding install receives a fresh generation.
pub(super) struct NativePluginRegistrationReplayGeneration {
    revision: u64,
    bridge_table: FrozenBridgeTable,
    manifest: NativePluginRegistrationManifest,
    component_type_ids: Vec<String>,
    granted_capabilities: Vec<String>,
    replay_context: Option<Arc<NativePluginRegistrationReplayBridgeContext>>,
    prepared_systems: Vec<PreparedNativePluginRegistrationSystem>,
}

/// The manifest-only parts of a system registration are parsed once per native generation.
/// Per-registry component availability remains checked during replay, where the registry is known.
struct PreparedNativePluginRegistrationSystem {
    stage: SystemStage,
    access_plan: Arc<NativeSystemAccessPlan>,
    bridge_interface: String,
    bridge_method: String,
    bridge_interface_slot: InterfaceSlot,
    bridge_method_slot: u32,
}

impl std::fmt::Debug for NativePluginRegistrationReplayGeneration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginRegistrationReplayGeneration")
            .field("revision", &self.revision)
            .field("systems", &self.manifest.systems.len())
            .field("component_type_ids", &self.component_type_ids.len())
            .field("granted_capabilities", &self.granted_capabilities.len())
            .field("has_replay_context", &self.replay_context.is_some())
            .field("prepared_systems", &self.prepared_systems.len())
            .finish()
    }
}

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
    InvalidSystemAccessAuthority {
        plugin_id: String,
        system_id: String,
        source: NativeSystemAccessAuthorityError,
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
            Self::InvalidSystemAccessAuthority {
                plugin_id,
                system_id,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` access was denied: {source}"
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
            Self::InvalidSystemAccessAuthority { source, .. } => Some(source),
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
    #[cfg(test)]
    pub(super) fn install_registration_replay_source_test_gate(
        &self,
    ) -> super::NativePluginLiveHostTestGate {
        self.registration_replay_source_test_hook.install()
    }

    #[cfg(test)]
    pub(super) fn install_registration_replay_before_cache_test_gate(
        &self,
    ) -> super::NativePluginLiveHostTestGate {
        self.registration_replay_before_cache_test_hook.install()
    }

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
        let generation =
            self.runtime_registration_replay_generation_result(plugin_id, lifecycle)?;
        let manifest = &generation.manifest;
        let known_component_ids = generation
            .component_type_ids
            .iter()
            .cloned()
            .chain(
                registry
                    .components()
                    .iter()
                    .map(|component| component.type_id.clone()),
            )
            .collect::<Vec<_>>();
        let access_authority = NativeSystemAccessAuthority::new(
            plugin_id,
            known_component_ids,
            manifest
                .resources
                .iter()
                .map(|resource| resource.id.clone()),
            generation.granted_capabilities.iter().cloned(),
        );
        let mut report = NativePluginRuntimeRegistrationReplayReport::default();
        if manifest.systems.is_empty() {
            return Ok(report);
        }
        let replay_context = generation
            .replay_context
            .as_deref()
            .expect("non-empty registration manifest must retain a replay context");
        debug_assert_eq!(manifest.systems.len(), generation.prepared_systems.len());
        for (system, prepared) in manifest.systems.iter().zip(&generation.prepared_systems) {
            let system_report = self.replay_runtime_registration_system(
                registry,
                plugin_id,
                system,
                prepared,
                replay_context,
                &access_authority,
            )?;
            report.registered_systems.push(system_report);
        }
        Ok(report)
    }

    /// Removes only the runtime plugin's cached generation. Callers change bindings before or
    /// after replacing the loaded entry, so the revision also rejects a builder that raced that
    /// transition and tries to publish the previous generation afterward.
    pub(super) fn invalidate_runtime_registration_replay_generation(&self, plugin_id: &str) {
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        let mut revisions = self
            .runtime_registration_replay_generation_revisions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let revision = revisions.get(&key).copied().unwrap_or_default();
        revisions.insert(key, revision.saturating_add(1));
        self.runtime_bridge_generations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&key);
        let mut generations = self
            .runtime_registration_replay_generations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        generations.remove(&key);
    }

    pub(super) fn runtime_bridge_generation_result(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeMethodResult<Arc<NativePluginRegistrationReplayBridgeContext>> {
        loop {
            if let Some(generation) = self.cached_runtime_bridge_generation(plugin_id, lifecycle) {
                return Ok(generation);
            }
            let _build_guard = self.lock_runtime_bridge_generation_build();
            if let Some(generation) = self.cached_runtime_bridge_generation(plugin_id, lifecycle) {
                return Ok(generation);
            }
            let revision = self.runtime_registration_replay_generation_revision(plugin_id);
            let generation = Arc::new(
                self.build_runtime_bridge_generation_result(plugin_id, lifecycle, revision)?,
            );
            if self.cache_runtime_bridge_generation(generation.clone(), plugin_id) {
                return Ok(generation);
            }
        }
    }

    fn lock_runtime_bridge_generation_build(&self) -> std::sync::MutexGuard<'_, ()> {
        self.runtime_bridge_generation_build_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn cached_runtime_bridge_generation(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Option<Arc<NativePluginRegistrationReplayBridgeContext>> {
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        let revisions = self
            .runtime_registration_replay_generation_revisions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let revision = revisions.get(&key).copied().unwrap_or_default();
        self.runtime_bridge_generations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&key)
            .filter(|generation| generation.matches(revision, lifecycle))
            .cloned()
    }

    fn cache_runtime_bridge_generation(
        &self,
        generation: Arc<NativePluginRegistrationReplayBridgeContext>,
        plugin_id: &str,
    ) -> bool {
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        let revisions = self
            .runtime_registration_replay_generation_revisions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if revisions.get(&key).copied().unwrap_or_default() != generation.revision {
            return false;
        }
        self.runtime_bridge_generations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(key, generation);
        true
    }

    fn runtime_registration_replay_generation_result(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginRegistrationReplayResult<Arc<NativePluginRegistrationReplayGeneration>> {
        loop {
            if let Some(generation) =
                self.cached_runtime_registration_replay_generation(plugin_id, lifecycle)
            {
                return Ok(generation);
            }
            let _build_guard = self.lock_runtime_registration_replay_generation_build();
            if let Some(generation) =
                self.cached_runtime_registration_replay_generation(plugin_id, lifecycle)
            {
                return Ok(generation);
            }
            let revision = self.runtime_registration_replay_generation_revision(plugin_id);
            let generation = Arc::new(
                self.build_runtime_registration_replay_generation(plugin_id, lifecycle, revision)?,
            );
            #[cfg(test)]
            self.registration_replay_before_cache_test_hook
                .pause_if_installed();
            if self.cache_runtime_registration_replay_generation(generation.clone(), plugin_id) {
                return Ok(generation);
            }
        }
    }

    fn lock_runtime_registration_replay_generation_build(&self) -> std::sync::MutexGuard<'_, ()> {
        self.runtime_registration_replay_generation_build_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn cached_runtime_registration_replay_generation(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Option<Arc<NativePluginRegistrationReplayGeneration>> {
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        // Keep the revision lock before the generation lock everywhere. Invalidation then removes
        // a stale entry atomically with respect to cache publication.
        let revisions = self
            .runtime_registration_replay_generation_revisions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let revision = revisions.get(&key).copied().unwrap_or_default();
        let generations = self
            .runtime_registration_replay_generations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        generations
            .get(&key)
            .filter(|generation| {
                generation.revision == revision
                    && generation
                        .bridge_table
                        .shares_storage_with(lifecycle.bridge_table())
            })
            .cloned()
    }

    fn runtime_registration_replay_generation_revision(&self, plugin_id: &str) -> u64 {
        self.runtime_registration_replay_generation_revisions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .copied()
            .unwrap_or_default()
    }

    fn cache_runtime_registration_replay_generation(
        &self,
        generation: Arc<NativePluginRegistrationReplayGeneration>,
        plugin_id: &str,
    ) -> bool {
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        let revisions = self
            .runtime_registration_replay_generation_revisions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if revisions.get(&key).copied().unwrap_or_default() != generation.revision {
            return false;
        }
        self.runtime_registration_replay_generations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(key, generation);
        true
    }

    fn build_runtime_registration_replay_generation(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        revision: u64,
    ) -> NativePluginRegistrationReplayResult<NativePluginRegistrationReplayGeneration> {
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginRegistrationReplayError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(
                || NativePluginRegistrationReplayError::RuntimePluginNotLoaded {
                    plugin_id: plugin_id.to_string(),
                },
            )?;
        let source = Self::runtime_registration_manifest_source(plugin_id, plugin)?;
        #[cfg(test)]
        self.registration_replay_source_test_hook
            .pause_if_installed();
        let manifest =
            NativePluginRegistrationManifest::from_toml(&source.text).map_err(|source| {
                NativePluginRegistrationReplayError::InvalidRegistrationManifest {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?;
        #[cfg(test)]
        self.registration_replay_context_build_counters
            .registration_manifest_parses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let replay_context = if manifest.systems.is_empty() {
            None
        } else {
            Some(
                self.runtime_bridge_generation_result(plugin_id, lifecycle)
                    .map_err(
                        |source| NativePluginRegistrationReplayError::BridgeCallScope {
                            plugin_id: plugin_id.to_string(),
                            source: source.to_string(),
                        },
                    )?,
            )
        };
        let prepared_systems = if let Some(replay_context) = replay_context.as_deref() {
            manifest
                .systems
                .iter()
                .map(|system| {
                    prepare_native_plugin_registration_system(
                        plugin_id,
                        system,
                        &manifest.capabilities,
                        replay_context,
                        lifecycle,
                    )
                })
                .collect::<NativePluginRegistrationReplayResult<Vec<_>>>()?
        } else {
            Vec::new()
        };
        #[cfg(test)]
        self.registration_replay_context_build_counters
            .registration_system_preparations
            .fetch_add(prepared_systems.len(), std::sync::atomic::Ordering::Relaxed);
        drop(loaded);
        Ok(NativePluginRegistrationReplayGeneration {
            revision,
            bridge_table: lifecycle.bridge_table().clone(),
            manifest,
            component_type_ids: source.component_type_ids,
            granted_capabilities: source.granted_capabilities,
            replay_context,
            prepared_systems,
        })
    }

    fn runtime_registration_manifest_source(
        plugin_id: &str,
        plugin: &LoadedNativePlugin,
    ) -> NativePluginRegistrationReplayResult<RuntimeRegistrationManifestSource> {
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
        let granted_capabilities = plugin
            .descriptor
            .as_ref()
            .map(|descriptor| granted_capabilities_for_entry(descriptor, PluginModuleKind::Runtime))
            .unwrap_or_default();
        let mut component_type_ids = Vec::new();
        for package_manifest in plugin
            .descriptor
            .as_ref()
            .and_then(|descriptor| descriptor.package_manifest.as_ref())
            .into_iter()
            .chain(
                plugin
                    .runtime_entry_report
                    .as_ref()
                    .and_then(|report| report.package_manifest.as_ref()),
            )
        {
            component_type_ids.extend(
                package_manifest
                    .components
                    .iter()
                    .map(|component| component.type_id.clone()),
            );
        }
        component_type_ids.sort();
        component_type_ids.dedup();
        Ok(RuntimeRegistrationManifestSource {
            text,
            component_type_ids,
            granted_capabilities,
        })
    }

    fn replay_runtime_registration_system(
        &self,
        registry: &mut RuntimeExtensionRegistry,
        plugin_id: &str,
        system: &NativePluginRegistrationSystem,
        prepared: &PreparedNativePluginRegistrationSystem,
        replay_context: &NativePluginRegistrationReplayBridgeContext,
        access_authority: &NativeSystemAccessAuthority,
    ) -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationSystemReplay> {
        access_authority
            .authorize(prepared.access_plan.as_ref())
            .map_err(|source| {
                NativePluginRegistrationReplayError::InvalidSystemAccessAuthority {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    source,
                }
            })?;
        register_bridge_replay_system(
            registry,
            plugin_id,
            system,
            prepared.stage,
            prepared.bridge_interface_slot,
            prepared.bridge_method_slot,
            replay_context.bridge_call_scope(),
            prepared.access_plan.clone(),
        )?;
        Ok(NativePluginRuntimeRegistrationSystemReplay {
            plugin_id: plugin_id.to_string(),
            module: system.module.clone(),
            system_id: system.id.clone(),
            stage: prepared.stage,
            order: system.order,
            bridge_interface: prepared.bridge_interface.clone(),
            bridge_method: prepared.bridge_method.clone(),
        })
    }
}

fn prepare_native_plugin_registration_system(
    plugin_id: &str,
    system: &NativePluginRegistrationSystem,
    manifest_capabilities: &[String],
    replay_context: &NativePluginRegistrationReplayBridgeContext,
    lifecycle: &RuntimePluginBridgeLifecycleState,
) -> NativePluginRegistrationReplayResult<PreparedNativePluginRegistrationSystem> {
    let stage = system.stage().map_err(|source| {
        NativePluginRegistrationReplayError::InvalidRegistrationSystem {
            plugin_id: plugin_id.to_string(),
            system_id: system.id.clone(),
            source,
        }
    })?;
    let access_plan = Arc::new(
        system
            .access_plan(manifest_capabilities)
            .map_err(
                |source| NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    source,
                },
            )?,
    );
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
    let bridge_method_slot = replay_context
        .method_slot_result(plugin_id, &bridge_interface, &bridge_method)
        .map_err(
            |source| NativePluginRegistrationReplayError::BridgeMethodSlot {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                bridge_interface: bridge_interface.clone(),
                bridge_method: bridge_method.clone(),
                source: source.to_string(),
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
    Ok(PreparedNativePluginRegistrationSystem {
        stage,
        access_plan,
        bridge_interface,
        bridge_method,
        bridge_interface_slot,
        bridge_method_slot,
    })
}

struct RuntimeRegistrationManifestSource {
    text: String,
    component_type_ids: Vec<String>,
    granted_capabilities: Vec<String>,
}

fn register_bridge_replay_system(
    registry: &mut RuntimeExtensionRegistry,
    plugin_id: &str,
    system: &NativePluginRegistrationSystem,
    stage: SystemStage,
    bridge_interface_slot: InterfaceSlot,
    bridge_method_slot: u32,
    bridge_call_scope: Arc<NativeHostBridgeCallScope>,
    access_plan: Arc<NativeSystemAccessPlan>,
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
    let affinity = access_plan.affinity();
    let mut builder = registry
        .register_external_native_system(
            owner,
            system.id.clone(),
            stage,
            affinity,
            move |world: &mut crate::scene::World| {
                access_plan
                    .compile(world)
                    .map_err(|error| error.to_string())
            },
            move || {
                let bridge_call_scope = Arc::clone(&bridge_call_scope);
                move || {
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
            },
        )
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
