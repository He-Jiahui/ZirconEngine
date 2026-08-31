use std::collections::HashMap;
#[cfg(test)]
use std::sync::atomic::Ordering;
use std::sync::{Arc, MutexGuard};

use crate::plugin::native::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeMethodBinding,
    NativeBridgeMethodManifestError, NativeHostBridgeCallScope,
};
use crate::plugin::{
    PluginModuleKind, PluginPackageManifest, RuntimeExtensionRegistryError,
    RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleState,
};

use super::super::loaded_native_plugin::NativePluginLibraryGenerationOwner;
use super::super::LoadedNativePlugin;
use super::keys::{live_key, NativePluginLiveRegistry};
use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
use super::registration_replay::NativePluginRegistrationReplayBridgeContext;
use super::reports::{
    NativePluginLiveHostBridgeLifecycleReport, NativePluginLiveHostBridgeReloadReport,
    NativePluginLiveHostCommand,
};
use super::NativePluginLiveHost;

pub(super) type NativePluginBridgeMethodResult<T> =
    std::result::Result<T, NativePluginBridgeMethodError>;

pub(super) struct ValidatedRuntimeBridgeMethodBindings {
    descriptors: Vec<crate::plugin::native::NativeBridgeMethodDescriptor>,
    method_slots: Arc<HashMap<String, HashMap<String, u32>>>,
    library_owner: NativePluginLibraryGenerationOwner,
}

impl ValidatedRuntimeBridgeMethodBindings {
    fn from_loaded_plugin(
        plugin: &LoadedNativePlugin,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> NativePluginBridgeMethodResult<Self> {
        let manifest = runtime_package_manifest(plugin).ok_or_else(|| {
            NativePluginBridgeMethodError::MissingPackageManifest {
                plugin_id: plugin.plugin_id.clone(),
            }
        })?;
        Self::from_manifest(manifest, plugin.library_generation_owner(), bindings)
    }

    fn from_rust_plugin(
        plugin: &LoadedNativePlugin,
        bindings: Vec<NativeBridgeMethodBinding>,
    ) -> NativePluginBridgeMethodResult<Self> {
        if let Some(binding) = bindings
            .iter()
            .find(|binding| binding.requires_loaded_generation_owner())
        {
            return Err(
                NativePluginBridgeMethodError::AbiBindingRequiresLoadedGenerationOwner {
                    plugin_id: plugin.plugin_id.clone(),
                    interface_id: binding.interface_id().to_string(),
                    method_name: binding.method_name().to_string(),
                },
            );
        }
        Self::from_loaded_plugin(plugin, bindings)
    }

    fn from_manifest(
        manifest: &PluginPackageManifest,
        library_owner: NativePluginLibraryGenerationOwner,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> NativePluginBridgeMethodResult<Self> {
        let descriptors = native_bridge_method_descriptors_from_manifest(manifest, bindings)
            .map_err(NativePluginBridgeMethodError::InvalidBridgeMethodManifest)?;
        let method_slots = Arc::new(
            manifest
                .provides_interfaces
                .iter()
                .map(|interface| {
                    (
                        interface.id.clone(),
                        interface
                            .methods
                            .iter()
                            .map(|method| (method.name.clone(), method.method_slot))
                            .collect(),
                    )
                })
                .collect(),
        );
        Ok(Self {
            descriptors,
            method_slots,
            library_owner,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.descriptors.len()
    }
}

impl std::fmt::Debug for ValidatedRuntimeBridgeMethodBindings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatedRuntimeBridgeMethodBindings")
            .field("methods", &self.descriptors.len())
            .finish()
    }
}

#[derive(Debug)]
pub(super) enum NativePluginBridgeMethodError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimePluginNotLoaded {
        plugin_id: String,
    },
    MissingDiscoveredBridgeMethodTable {
        plugin_id: String,
    },
    MissingPackageManifest {
        plugin_id: String,
    },
    MissingInstalledBridgeMethodBindings {
        plugin_id: String,
    },
    AbiBindingRequiresLoadedGenerationOwner {
        plugin_id: String,
        interface_id: String,
        method_name: String,
    },
    InvalidBridgeMethodManifest(NativeBridgeMethodManifestError),
    BridgeCallScope(RuntimeExtensionRegistryError),
    BridgeLifecycleRejected {
        diagnostic: String,
    },
    MissingDeclaredBridgeMethod {
        plugin_id: String,
        interface_id: String,
        method_name: String,
    },
}

impl std::fmt::Display for NativePluginBridgeMethodError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLock(error) => write!(formatter, "{error}"),
            Self::RuntimePluginNotLoaded { plugin_id } => write!(
                formatter,
                "plugin {plugin_id} is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
            ),
            Self::MissingDiscoveredBridgeMethodTable { plugin_id } => write!(
                formatter,
                "runtime plugin {plugin_id} exposes no native bridge method table"
            ),
            Self::MissingPackageManifest { plugin_id } => {
                write!(
                    formatter,
                    "runtime plugin {plugin_id} has no package manifest"
                )
            }
            Self::MissingInstalledBridgeMethodBindings { plugin_id } => write!(
                formatter,
                "runtime plugin {plugin_id} has no installed native bridge method bindings"
            ),
            Self::AbiBindingRequiresLoadedGenerationOwner {
                plugin_id,
                interface_id,
                method_name,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} cannot safely install ownerless ABI bridge method `{interface_id}.{method_name}`; install callbacks from its loaded native generation"
            ),
            Self::InvalidBridgeMethodManifest(error) => write!(formatter, "{error}"),
            Self::BridgeCallScope(error) => write!(formatter, "{error}"),
            Self::BridgeLifecycleRejected { diagnostic } => write!(formatter, "{diagnostic}"),
            Self::MissingDeclaredBridgeMethod {
                plugin_id,
                interface_id,
                method_name,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} package manifest does not declare bridge method `{interface_id}.{method_name}`"
            ),
        }
    }
}

impl std::error::Error for NativePluginBridgeMethodError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LiveHostLock(error) => Some(error),
            Self::InvalidBridgeMethodManifest(error) => Some(error),
            Self::BridgeCallScope(error) => Some(error),
            Self::RuntimePluginNotLoaded { .. }
            | Self::MissingDiscoveredBridgeMethodTable { .. }
            | Self::MissingPackageManifest { .. }
            | Self::MissingInstalledBridgeMethodBindings { .. }
            | Self::AbiBindingRequiresLoadedGenerationOwner { .. }
            | Self::BridgeLifecycleRejected { .. }
            | Self::MissingDeclaredBridgeMethod { .. } => None,
        }
    }
}

impl NativePluginLiveHost {
    pub fn install_discovered_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<usize, String> {
        self.install_discovered_runtime_bridge_method_bindings_result(plugin_id)
            .map_err(|error| error.to_string())
    }

    pub(super) fn install_discovered_runtime_bridge_method_bindings_result(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> NativePluginBridgeMethodResult<usize> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginBridgeMethodError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| NativePluginBridgeMethodError::RuntimePluginNotLoaded {
                plugin_id: plugin_id.to_string(),
            })?;
        let validated_bindings = discovered_runtime_bridge_method_bindings_result(plugin)?
            .ok_or_else(
                || NativePluginBridgeMethodError::MissingDiscoveredBridgeMethodTable {
                    plugin_id: plugin_id.to_string(),
                },
            )?;
        let binding_count = validated_bindings.len();
        self.publish_runtime_bridge_method_bindings_under_loaded_lock_result(
            &loaded,
            plugin_id,
            Some(validated_bindings),
        )?;
        self.invalidate_runtime_registration_replay_generation(plugin_id);
        drop(loaded);
        Ok(binding_count)
    }

    pub fn install_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> Result<(), String> {
        self.install_runtime_bridge_method_bindings_result(plugin_id, bindings)
            .map_err(|error| error.to_string())
    }

    pub(super) fn install_runtime_bridge_method_bindings_result(
        &self,
        plugin_id: impl AsRef<str>,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> NativePluginBridgeMethodResult<()> {
        let plugin_id = plugin_id.as_ref();
        // Caller-provided iterators may execute arbitrary code. Materialize them before taking
        // the non-reentrant live-host mutex, then validate and publish against one loaded entry.
        let bindings = bindings.into_iter().collect::<Vec<_>>();
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginBridgeMethodError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| NativePluginBridgeMethodError::RuntimePluginNotLoaded {
                plugin_id: plugin_id.to_string(),
            })?;
        let validated_bindings =
            ValidatedRuntimeBridgeMethodBindings::from_rust_plugin(plugin, bindings)?;
        self.publish_runtime_bridge_method_bindings_under_loaded_lock_result(
            &loaded,
            plugin_id,
            Some(validated_bindings),
        )?;
        self.invalidate_runtime_registration_replay_generation(plugin_id);
        drop(loaded);
        Ok(())
    }

    pub fn clear_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<bool, String> {
        let plugin_id = plugin_id.as_ref();
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        let loaded = lock_loaded_native_plugins(&self.loaded).map_err(|error| error.to_string())?;
        let mut bindings = self.lock_runtime_bridge_method_bindings();
        let removed = bindings.remove(&key).is_some();
        drop(bindings);
        if removed {
            self.invalidate_runtime_registration_replay_generation(plugin_id);
        }
        drop(loaded);
        Ok(removed)
    }

    pub fn runtime_bridge_call_scope_from_installed_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativeHostBridgeCallScope, String> {
        self.runtime_bridge_call_scope_from_installed_bindings_result(plugin_id, lifecycle)
            .map_err(|error| error.to_string())
    }

    pub(super) fn runtime_bridge_call_scope_from_installed_bindings_result(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeMethodResult<NativeHostBridgeCallScope> {
        let plugin_id = plugin_id.as_ref();
        let generation = self.runtime_bridge_generation_result(plugin_id, lifecycle)?;
        Ok(generation.bridge_call_scope.as_ref().clone())
    }

    pub(super) fn build_runtime_bridge_generation_result(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        revision: u64,
    ) -> NativePluginBridgeMethodResult<NativePluginRegistrationReplayBridgeContext> {
        // Installation validates and projects the manifest exactly once. A cold lifecycle-table
        // generation only borrows those descriptors and shares the prebuilt name-to-slot map.
        let (bridge_call_scope, method_slots) = {
            let installed_bindings = self.lock_runtime_bridge_method_bindings();
            let validated_bindings = installed_bindings
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(|| {
                    NativePluginBridgeMethodError::MissingInstalledBridgeMethodBindings {
                        plugin_id: plugin_id.to_string(),
                    }
                })?;
            let bridge_call_scope = Arc::new(
                NativeHostBridgeCallScope::from_method_descriptor_refs_with_owner(
                    lifecycle.bridge_table().clone(),
                    validated_bindings.descriptors.iter(),
                    Some(validated_bindings.library_owner.clone()),
                )
                .map_err(NativePluginBridgeMethodError::BridgeCallScope)?,
            );
            (bridge_call_scope, validated_bindings.method_slots.clone())
        };
        #[cfg(test)]
        self.registration_replay_context_build_counters
            .bridge_call_scope_builds
            .fetch_add(1, Ordering::Relaxed);
        Ok(NativePluginRegistrationReplayBridgeContext {
            revision,
            bridge_table: lifecycle.bridge_table().clone(),
            method_slots,
            bridge_call_scope,
        })
    }

    pub fn reload_runtime_bridge_provider_and_scope_from_installed_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostBridgeReloadReport, String> {
        self.reload_runtime_bridge_provider_and_scope_from_installed_bindings_result(
            plugin_id, lifecycle,
        )
        .map_err(|error| error.to_string())
    }

    pub(super) fn reload_runtime_bridge_provider_and_scope_from_installed_bindings_result(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeMethodResult<NativePluginLiveHostBridgeReloadReport> {
        let plugin_id = plugin_id.as_ref();
        let event = RuntimePluginBridgeLifecycleEvent::reload_provider(plugin_id);
        let bridge_lifecycle_report = NativePluginLiveHostBridgeLifecycleReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            command: NativePluginLiveHostCommand::HotReload,
            event: event.clone(),
            outcome: lifecycle.apply_provider_lifecycle_event(event),
        };
        if !bridge_lifecycle_report.is_applied() {
            return Err(NativePluginBridgeMethodError::BridgeLifecycleRejected {
                diagnostic: bridge_lifecycle_report.diagnostic(),
            });
        }

        let bridge_call_scope =
            self.runtime_bridge_call_scope_from_installed_bindings_result(plugin_id, lifecycle)?;
        let mut report = NativePluginLiveHostBridgeReloadReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            command: NativePluginLiveHostCommand::HotReload,
            bridge_lifecycle_report,
            bridge_call_scope,
            diagnostics: Vec::new(),
        };
        report
            .diagnostics
            .push(report.bridge_lifecycle_report.diagnostic());
        report.diagnostics.push(report.diagnostic());
        report.diagnostics.sort_unstable();
        report.diagnostics.dedup();
        Ok(report)
    }

    #[cfg(test)]
    pub(super) fn installed_runtime_bridge_method_binding_count(
        &self,
        plugin_id: &str,
    ) -> Result<usize, String> {
        self.installed_runtime_bridge_method_binding_count_result(plugin_id)
            .map_err(|error| error.to_string())
    }

    #[cfg(test)]
    pub(super) fn installed_runtime_bridge_method_binding_count_result(
        &self,
        plugin_id: &str,
    ) -> NativePluginBridgeMethodResult<usize> {
        self.lock_runtime_bridge_method_bindings()
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .map(ValidatedRuntimeBridgeMethodBindings::len)
            .ok_or_else(
                || NativePluginBridgeMethodError::MissingInstalledBridgeMethodBindings {
                    plugin_id: plugin_id.to_string(),
                },
            )
    }

    pub(super) fn publish_runtime_bridge_method_bindings_under_loaded_lock_result(
        &self,
        _loaded: &MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>,
        plugin_id: &str,
        bindings: Option<ValidatedRuntimeBridgeMethodBindings>,
    ) -> NativePluginBridgeMethodResult<()> {
        let mut installed_bindings = self.lock_runtime_bridge_method_bindings();
        match bindings {
            Some(bindings) if bindings.len() != 0 => {
                #[cfg(test)]
                self.registration_replay_context_build_counters
                    .method_lookup_builds
                    .fetch_add(1, Ordering::Relaxed);
                installed_bindings.insert(live_key(PluginModuleKind::Runtime, plugin_id), bindings);
            }
            Some(_) | None => {
                installed_bindings.remove(&live_key(PluginModuleKind::Runtime, plugin_id));
            }
        }
        drop(installed_bindings);
        Ok(())
    }

    fn lock_runtime_bridge_method_bindings(
        &self,
    ) -> MutexGuard<'_, NativePluginLiveRegistry<ValidatedRuntimeBridgeMethodBindings>> {
        self.runtime_bridge_method_bindings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn runtime_bridge_method_slot_result(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        interface_id: &str,
        method_name: &str,
    ) -> NativePluginBridgeMethodResult<u32> {
        self.runtime_bridge_generation_result(plugin_id, lifecycle)?
            .method_slot_result(plugin_id, interface_id, method_name)
    }
}

pub(super) fn discovered_runtime_bridge_method_bindings_result(
    plugin: &LoadedNativePlugin,
) -> NativePluginBridgeMethodResult<Option<ValidatedRuntimeBridgeMethodBindings>> {
    let Some(report) = plugin.runtime_entry_report.as_ref() else {
        return Ok(None);
    };
    if report.bridge_method_bindings.is_empty() {
        return Ok(None);
    }
    ValidatedRuntimeBridgeMethodBindings::from_loaded_plugin(
        plugin,
        report.bridge_method_bindings.iter().cloned(),
    )
    .map(Some)
}

pub(super) fn discovered_runtime_bridge_method_binding_diagnostics(
    plugin_id: &str,
    binding_count: usize,
) -> String {
    format!(
        "native.live_host.bridge_bindings_discovered: Runtime plugin `{plugin_id}` installed {} bridge method(s)",
        binding_count
    )
}

pub(super) fn discovered_runtime_bridge_method_binding_error_diagnostic(
    plugin_id: &str,
    error: &impl std::fmt::Display,
) -> String {
    format!(
        "native.live_host.bridge_bindings_discovery_failed: Runtime plugin `{plugin_id}` bridge method table rejected: {error}"
    )
}

fn runtime_package_manifest(
    plugin: &super::super::LoadedNativePlugin,
) -> Option<&PluginPackageManifest> {
    plugin
        .runtime_entry_report
        .as_ref()
        .and_then(|report| report.package_manifest.as_ref())
        .or_else(|| {
            plugin
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.package_manifest.as_ref())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_live_host_bridge_method_bindings_recover_poisoned_lock() {
        let host = NativePluginLiveHost::default();
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _bindings = host.runtime_bridge_method_bindings.lock().unwrap();
            panic!("poison native live-host bridge method bindings");
        }));
        assert!(poison.is_err());

        assert!(!host
            .clear_runtime_bridge_method_bindings("physics")
            .expect("poisoned binding lock should recover for clear"));
        assert!(matches!(
            host.installed_runtime_bridge_method_binding_count("physics"),
            Err(message) if message == "runtime plugin physics has no installed native bridge method bindings"
        ));
    }
}
