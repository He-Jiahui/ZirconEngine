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

use super::super::loaded_native_plugin::{
    NativePluginCallbackLease, NativePluginCallbackLeaseError,
};

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
    InvalidBridgeMethodManifest(NativeBridgeMethodManifestError),
    BridgeCallScope(RuntimeExtensionRegistryError),
    CallbackOwner {
        plugin_id: String,
        source: NativePluginCallbackLeaseError,
    },
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
            Self::InvalidBridgeMethodManifest(error) => write!(formatter, "{error}"),
            Self::BridgeCallScope(error) => write!(formatter, "{error}"),
            Self::CallbackOwner { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} stable callback owner rejected: {source}"
            ),
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
            Self::CallbackOwner { source, .. } => Some(source),
            Self::RuntimePluginNotLoaded { .. }
            | Self::MissingDiscoveredBridgeMethodTable { .. }
            | Self::MissingPackageManifest { .. }
            | Self::MissingInstalledBridgeMethodBindings { .. }
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
        let (bindings, _callback_owner) = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginBridgeMethodError::LiveHostLock)?;
            let plugin = loaded
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(|| NativePluginBridgeMethodError::RuntimePluginNotLoaded {
                    plugin_id: plugin_id.to_string(),
                })?;
            let callback_owner = plugin.callback_owner_lease().map_err(|source| {
                NativePluginBridgeMethodError::CallbackOwner {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?;
            let bindings =
                discovered_runtime_bridge_method_bindings_result(plugin)?.ok_or_else(|| {
                    NativePluginBridgeMethodError::MissingDiscoveredBridgeMethodTable {
                        plugin_id: plugin_id.to_string(),
                    }
                })?;
            (bindings, callback_owner)
        };
        let binding_count = bindings.len();
        self.replace_runtime_bridge_method_bindings_result(plugin_id, Some(bindings))?;
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
        let bindings = bindings.into_iter().collect::<Vec<_>>();
        let (manifest, _callback_owner) =
            self.loaded_runtime_package_manifest_and_callback_owner_result(plugin_id)?;
        native_bridge_method_descriptors_from_manifest(&manifest, bindings.clone())
            .map_err(NativePluginBridgeMethodError::InvalidBridgeMethodManifest)?;

        self.replace_runtime_bridge_method_bindings_result(plugin_id, Some(bindings))?;
        Ok(())
    }

    pub fn clear_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<bool, String> {
        let plugin_id = plugin_id.as_ref();
        let key = live_key(PluginModuleKind::Runtime, plugin_id);
        let mut bindings = self.lock_runtime_bridge_method_bindings();
        let removed = bindings.remove(&key).is_some();
        drop(bindings);
        if removed {
            self.invalidate_runtime_registration_replay_generation(plugin_id);
        }
        Ok(removed)
    }

    pub fn runtime_bridge_call_scope_from_loaded_manifest(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> Result<NativeHostBridgeCallScope, String> {
        self.runtime_bridge_call_scope_from_loaded_manifest_result(plugin_id, lifecycle, bindings)
            .map_err(|error| error.to_string())
    }

    pub(super) fn runtime_bridge_call_scope_from_loaded_manifest_result(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> NativePluginBridgeMethodResult<NativeHostBridgeCallScope> {
        let plugin_id = plugin_id.as_ref();
        let (manifest, callback_owner) =
            self.loaded_runtime_package_manifest_and_callback_owner_result(plugin_id)?;
        Self::runtime_bridge_call_scope_from_manifest_and_owner_result(
            lifecycle,
            bindings,
            manifest,
            callback_owner,
        )
    }

    fn runtime_bridge_call_scope_from_manifest_and_owner_result(
        lifecycle: &RuntimePluginBridgeLifecycleState,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
        manifest: PluginPackageManifest,
        callback_owner: NativePluginCallbackLease,
    ) -> NativePluginBridgeMethodResult<NativeHostBridgeCallScope> {
        let descriptors = native_bridge_method_descriptors_from_manifest(&manifest, bindings)
            .map_err(NativePluginBridgeMethodError::InvalidBridgeMethodManifest)?;
        NativeHostBridgeCallScope::from_method_descriptors_with_owner(
            lifecycle.bridge_table().clone(),
            descriptors,
            Some(callback_owner),
        )
        .map_err(NativePluginBridgeMethodError::BridgeCallScope)
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
        // Acquire the library owner before copying installed function pointers.
        // Hot reload/unload must observe this active owner and reject the
        // transition until the resulting bridge scope is fully constructed.
        let (manifest, callback_owner) =
            self.loaded_runtime_package_manifest_and_callback_owner_result(plugin_id)?;
        let bindings = self.installed_runtime_bridge_method_bindings_result(plugin_id)?;
        Self::runtime_bridge_call_scope_from_manifest_and_owner_result(
            lifecycle,
            bindings,
            manifest,
            callback_owner,
        )
    }

    pub(super) fn runtime_registration_replay_bridge_context_result(
        &self,
        plugin_id: &str,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeMethodResult<NativePluginRegistrationReplayBridgeContext> {
        // Keep the package manifest borrowed from its loaded generation while the callback lease
        // is acquired. The resulting descriptors and slots are the only owned data retained by
        // the replay generation, avoiding a full package-manifest or binding-vector clone.
        let (descriptors, method_slots, callback_owner) = {
            let loaded = lock_loaded_native_plugins(&self.loaded)
                .map_err(NativePluginBridgeMethodError::LiveHostLock)?;
            let plugin = loaded
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(|| NativePluginBridgeMethodError::RuntimePluginNotLoaded {
                    plugin_id: plugin_id.to_string(),
                })?;
            let manifest = runtime_package_manifest(plugin).ok_or_else(|| {
                NativePluginBridgeMethodError::MissingPackageManifest {
                    plugin_id: plugin_id.to_string(),
                }
            })?;
            let callback_owner = plugin.callback_owner_lease().map_err(|source| {
                NativePluginBridgeMethodError::CallbackOwner {
                    plugin_id: plugin_id.to_string(),
                    source,
                }
            })?;
            #[cfg(test)]
            self.registration_replay_context_build_counters
                .package_manifest_snapshots
                .fetch_add(1, Ordering::Relaxed);
            let installed_bindings = self.lock_runtime_bridge_method_bindings();
            let bindings = installed_bindings
                .get(&live_key(PluginModuleKind::Runtime, plugin_id))
                .ok_or_else(|| {
                    NativePluginBridgeMethodError::MissingInstalledBridgeMethodBindings {
                        plugin_id: plugin_id.to_string(),
                    }
                })?;
            #[cfg(test)]
            self.registration_replay_context_build_counters
                .binding_snapshots
                .fetch_add(1, Ordering::Relaxed);
            let descriptors =
                native_bridge_method_descriptors_from_manifest(manifest, bindings.iter().cloned())
                    .map_err(NativePluginBridgeMethodError::InvalidBridgeMethodManifest)?;
            let method_slots = manifest
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
                .collect();
            #[cfg(test)]
            self.registration_replay_context_build_counters
                .method_lookup_builds
                .fetch_add(1, Ordering::Relaxed);
            (descriptors, method_slots, callback_owner)
        };
        let bridge_call_scope = Arc::new(
            NativeHostBridgeCallScope::from_method_descriptors_with_owner(
                lifecycle.bridge_table().clone(),
                descriptors,
                Some(callback_owner),
            )
            .map_err(NativePluginBridgeMethodError::BridgeCallScope)?,
        );
        #[cfg(test)]
        self.registration_replay_context_build_counters
            .bridge_call_scope_builds
            .fetch_add(1, Ordering::Relaxed);
        Ok(NativePluginRegistrationReplayBridgeContext {
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
        report.diagnostics.sort();
        report.diagnostics.dedup();
        Ok(report)
    }

    fn loaded_runtime_package_manifest_required_result(
        &self,
        plugin_id: &str,
    ) -> NativePluginBridgeMethodResult<PluginPackageManifest> {
        self.loaded_runtime_package_manifest_result(plugin_id)?
            .ok_or_else(|| NativePluginBridgeMethodError::MissingPackageManifest {
                plugin_id: plugin_id.to_string(),
            })
    }

    fn loaded_runtime_package_manifest_and_callback_owner_result(
        &self,
        plugin_id: &str,
    ) -> NativePluginBridgeMethodResult<(PluginPackageManifest, NativePluginCallbackLease)> {
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginBridgeMethodError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| NativePluginBridgeMethodError::RuntimePluginNotLoaded {
                plugin_id: plugin_id.to_string(),
            })?;
        let manifest = runtime_package_manifest(plugin).cloned().ok_or_else(|| {
            NativePluginBridgeMethodError::MissingPackageManifest {
                plugin_id: plugin_id.to_string(),
            }
        })?;
        let callback_owner = plugin.callback_owner_lease().map_err(|source| {
            NativePluginBridgeMethodError::CallbackOwner {
                plugin_id: plugin_id.to_string(),
                source,
            }
        })?;
        Ok((manifest, callback_owner))
    }

    fn installed_runtime_bridge_method_bindings(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<NativeBridgeMethodBinding>, String> {
        self.installed_runtime_bridge_method_bindings_result(plugin_id)
            .map_err(|error| error.to_string())
    }

    fn installed_runtime_bridge_method_bindings_result(
        &self,
        plugin_id: &str,
    ) -> NativePluginBridgeMethodResult<Vec<NativeBridgeMethodBinding>> {
        self.lock_runtime_bridge_method_bindings()
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .cloned()
            .ok_or_else(
                || NativePluginBridgeMethodError::MissingInstalledBridgeMethodBindings {
                    plugin_id: plugin_id.to_string(),
                },
            )
    }

    pub(super) fn replace_runtime_bridge_method_bindings(
        &self,
        plugin_id: &str,
        bindings: Option<Vec<NativeBridgeMethodBinding>>,
    ) -> Result<(), String> {
        self.replace_runtime_bridge_method_bindings_result(plugin_id, bindings)
            .map_err(|error| error.to_string())
    }

    pub(super) fn replace_runtime_bridge_method_bindings_result(
        &self,
        plugin_id: &str,
        bindings: Option<Vec<NativeBridgeMethodBinding>>,
    ) -> NativePluginBridgeMethodResult<()> {
        let mut installed_bindings = self.lock_runtime_bridge_method_bindings();
        match bindings {
            Some(bindings) if !bindings.is_empty() => {
                installed_bindings.insert(live_key(PluginModuleKind::Runtime, plugin_id), bindings);
            }
            Some(_) | None => {
                installed_bindings.remove(&live_key(PluginModuleKind::Runtime, plugin_id));
            }
        }
        drop(installed_bindings);
        // Registered callbacks keep their old `Arc` generation alive; only future replay must
        // observe the replacement binding table.
        self.invalidate_runtime_registration_replay_generation(plugin_id);
        Ok(())
    }

    fn lock_runtime_bridge_method_bindings(
        &self,
    ) -> MutexGuard<'_, NativePluginLiveRegistry<Vec<NativeBridgeMethodBinding>>> {
        self.runtime_bridge_method_bindings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn runtime_bridge_method_slot(
        &self,
        plugin_id: &str,
        interface_id: &str,
        method_name: &str,
    ) -> Result<u32, String> {
        self.runtime_bridge_method_slot_result(plugin_id, interface_id, method_name)
            .map_err(|error| error.to_string())
    }

    pub(super) fn runtime_bridge_method_slot_result(
        &self,
        plugin_id: &str,
        interface_id: &str,
        method_name: &str,
    ) -> NativePluginBridgeMethodResult<u32> {
        let manifest = self.loaded_runtime_package_manifest_required_result(plugin_id)?;
        for interface in &manifest.provides_interfaces {
            if interface.id != interface_id {
                continue;
            }
            for method in &interface.methods {
                if method.name == method_name {
                    return Ok(method.method_slot);
                }
            }
        }
        Err(NativePluginBridgeMethodError::MissingDeclaredBridgeMethod {
            plugin_id: plugin_id.to_string(),
            interface_id: interface_id.to_string(),
            method_name: method_name.to_string(),
        })
    }

    fn loaded_runtime_package_manifest_result(
        &self,
        plugin_id: &str,
    ) -> NativePluginBridgeMethodResult<Option<PluginPackageManifest>> {
        let loaded = lock_loaded_native_plugins(&self.loaded)
            .map_err(NativePluginBridgeMethodError::LiveHostLock)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| NativePluginBridgeMethodError::RuntimePluginNotLoaded {
                plugin_id: plugin_id.to_string(),
            })?;
        Ok(runtime_package_manifest(plugin).cloned())
    }
}

pub(super) fn discovered_runtime_bridge_method_bindings(
    plugin: &super::super::LoadedNativePlugin,
) -> Result<Option<Vec<NativeBridgeMethodBinding>>, String> {
    discovered_runtime_bridge_method_bindings_result(plugin).map_err(|error| error.to_string())
}

pub(super) fn discovered_runtime_bridge_method_bindings_result(
    plugin: &super::super::LoadedNativePlugin,
) -> NativePluginBridgeMethodResult<Option<Vec<NativeBridgeMethodBinding>>> {
    let Some(report) = plugin.runtime_entry_report.as_ref() else {
        return Ok(None);
    };
    if report.bridge_method_bindings.is_empty() {
        return Ok(None);
    }
    let manifest = runtime_package_manifest(plugin).ok_or_else(|| {
        NativePluginBridgeMethodError::MissingPackageManifest {
            plugin_id: plugin.plugin_id.clone(),
        }
    })?;
    native_bridge_method_descriptors_from_manifest(manifest, report.bridge_method_bindings.clone())
        .map_err(NativePluginBridgeMethodError::InvalidBridgeMethodManifest)?;
    Ok(Some(report.bridge_method_bindings.clone()))
}

pub(super) fn discovered_runtime_bridge_method_binding_diagnostics(
    plugin_id: &str,
    bindings: &[NativeBridgeMethodBinding],
) -> String {
    format!(
        "native.live_host.bridge_bindings_discovered: Runtime plugin `{plugin_id}` installed {} bridge method(s)",
        bindings.len()
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
    use crate::plugin::native::{NativeBridgeCall, NativeBridgeMethodFn};
    use zircon_runtime_interface::{ZrByteSlice, ZrStatus, ZrStatusCode};

    #[test]
    fn native_live_host_bridge_method_bindings_recover_poisoned_lock() {
        let host = NativePluginLiveHost::default();
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut bindings = host.runtime_bridge_method_bindings.lock().unwrap();
            bindings.insert(
                live_key(PluginModuleKind::Runtime, "physics"),
                vec![bridge_binding("sample_count")],
            );
            panic!("poison native live-host bridge method bindings");
        }));
        assert!(poison.is_err());

        let installed = host
            .installed_runtime_bridge_method_bindings("physics")
            .expect("poisoned binding lock should recover for reads");
        assert_eq!(installed.len(), 1);

        assert!(host
            .clear_runtime_bridge_method_bindings("physics")
            .expect("poisoned binding lock should recover for clear"));
        assert!(matches!(
            host.installed_runtime_bridge_method_bindings("physics"),
            Err(message) if message == "runtime plugin physics has no installed native bridge method bindings"
        ));

        host.replace_runtime_bridge_method_bindings(
            "physics",
            Some(vec![bridge_binding("resample_count")]),
        )
        .expect("poisoned binding lock should recover for replace");
        assert_eq!(
            host.installed_runtime_bridge_method_bindings("physics")
                .expect("replaced binding should be readable after poison")
                .len(),
            1
        );
    }

    fn bridge_binding(method_name: &str) -> NativeBridgeMethodBinding {
        NativeBridgeMethodBinding::new(
            "test.native.live_host.bridge.v1",
            method_name,
            NativeBridgeMethodFn::from_rust(poisoned_bridge_method),
        )
    }

    fn poisoned_bridge_method(_call: NativeBridgeCall) -> ZrStatus {
        ZrStatus::new(ZrStatusCode::Ok, ZrByteSlice::empty())
    }
}
