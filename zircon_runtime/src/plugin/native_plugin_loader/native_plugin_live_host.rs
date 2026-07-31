#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use super::{
    LoadedNativePlugin, NativeBridgeMethodBinding, NativePluginCallbackDiagnostics,
    NativePluginLoader,
};
use crate::plugin::PluginModuleKind;

mod bridge_lifecycle;
mod bridge_methods;
mod diagnostics;
mod hot_reload;
mod hot_update_application;
mod keys;
mod lifecycle;
mod loading;
mod registration_replay;
mod reports;
mod runtime_behavior;

use keys::NativePluginLiveRegistry;
use registration_replay::NativePluginRegistrationReplayGeneration;

#[cfg(test)]
use super::{
    NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
#[cfg(test)]
use bridge_lifecycle::NativePluginBridgeLifecycleError;
#[cfg(test)]
use bridge_methods::NativePluginBridgeMethodError;
#[cfg(test)]
use diagnostics::{diagnostics_from_behavior_report, NativePluginBehaviorDiagnosticError};
#[cfg(test)]
use hot_reload::{
    restore_runtime_snapshot, NativePluginHotReloadError, NativePluginHotReloadState,
};
#[cfg(test)]
use keys::live_key;
#[cfg(test)]
use lifecycle::{load_for_module_kind, NativePluginLiveHostLifecycleError};
#[cfg(test)]
use loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};
#[cfg(test)]
use registration_replay::NativePluginRegistrationReplayError;
pub use reports::{
    NativePluginLiveHostBridgeLifecycleReport, NativePluginLiveHostBridgeReloadReport,
    NativePluginLiveHostCommand, NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeBehaviorDescriptor,
    NativePluginRuntimeCommandDispatchReport, NativePluginRuntimeDeltaHotUpdateReport,
    NativePluginRuntimeDeltaHotUpdateRequest, NativePluginRuntimeHotUpdateReport,
    NativePluginRuntimePlayModeExitReport, NativePluginRuntimePlayModeSnapshot,
    NativePluginRuntimePluginState, NativePluginRuntimeRegistrationReplayReport,
    NativePluginRuntimeRegistrationSystemReplay, NativePluginRuntimeStateRestoreReport,
    NativePluginRuntimeStateSnapshot, NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND,
    NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND,
};
#[cfg(test)]
use runtime_behavior::NativePluginRuntimeBehaviorError;
#[cfg(test)]
use runtime_behavior::{allow_missing_unload_callback_to_drop_handle, unload_behavior};

#[derive(Debug, Default)]
pub struct NativePluginLiveHost {
    loader: NativePluginLoader,
    loaded: ObservedLoadedNativePlugins,
    runtime_bridge_method_bindings: Mutex<NativePluginLiveRegistry<Vec<NativeBridgeMethodBinding>>>,
    // A generation captures the parsed manifest and the callback owner that backs its slots.
    // The revision map prevents a late builder from publishing an older load or binding generation.
    runtime_registration_replay_generations:
        Mutex<NativePluginLiveRegistry<Arc<NativePluginRegistrationReplayGeneration>>>,
    runtime_registration_replay_generation_revisions: Mutex<NativePluginLiveRegistry<u64>>,
    #[cfg(test)]
    registration_replay_context_build_counters: RegistrationReplayContextBuildCounters,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativePluginLiveHostDiagnostics {
    pub loaded_lock_acquisitions: u64,
    pub total_loaded_lock_wait_ns: u64,
    pub max_loaded_lock_wait_ns: u64,
}

#[derive(Debug, Default)]
struct ObservedLoadedNativePlugins {
    entries: Mutex<NativePluginLiveRegistry<LoadedNativePlugin>>,
    lock_acquisitions: AtomicU64,
    total_lock_wait_ns: AtomicU64,
    max_lock_wait_ns: AtomicU64,
}

impl ObservedLoadedNativePlugins {
    fn lock(
        &self,
    ) -> std::sync::LockResult<MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>> {
        let wait_started = Instant::now();
        let result = self.entries.lock();
        let wait_ns = wait_started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        self.lock_acquisitions.fetch_add(1, Ordering::Relaxed);
        self.total_lock_wait_ns
            .fetch_add(wait_ns, Ordering::Relaxed);
        self.max_lock_wait_ns.fetch_max(wait_ns, Ordering::Relaxed);
        result
    }

    fn diagnostics(&self) -> NativePluginLiveHostDiagnostics {
        NativePluginLiveHostDiagnostics {
            loaded_lock_acquisitions: self.lock_acquisitions.load(Ordering::Relaxed),
            total_loaded_lock_wait_ns: self.total_lock_wait_ns.load(Ordering::Relaxed),
            max_loaded_lock_wait_ns: self.max_lock_wait_ns.load(Ordering::Relaxed),
        }
    }
}

impl NativePluginLiveHost {
    pub fn live_host_diagnostics(&self) -> NativePluginLiveHostDiagnostics {
        self.loaded.diagnostics()
    }

    pub fn plugin_callback_diagnostics(
        &self,
        plugin_id: impl AsRef<str>,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginCallbackDiagnostics, String> {
        let plugin_id = plugin_id.as_ref();
        let loaded =
            loading::lock_loaded_native_plugins(&self.loaded).map_err(|error| error.to_string())?;
        loaded
            .get(&keys::live_key(module_kind, plugin_id))
            .map(LoadedNativePlugin::callback_diagnostics)
            .ok_or_else(|| {
                format!(
                    "plugin {plugin_id} is not loaded in the {} live host",
                    keys::module_kind_label(module_kind)
                )
            })
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
struct RegistrationReplayContextBuildCounters {
    registration_manifest_parses: AtomicUsize,
    registration_system_preparations: AtomicUsize,
    package_manifest_snapshots: AtomicUsize,
    binding_snapshots: AtomicUsize,
    method_lookup_builds: AtomicUsize,
    bridge_call_scope_builds: AtomicUsize,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RegistrationReplayContextBuildCounts {
    registration_manifest_parses: usize,
    registration_system_preparations: usize,
    package_manifest_snapshots: usize,
    binding_snapshots: usize,
    method_lookup_builds: usize,
    bridge_call_scope_builds: usize,
}

#[cfg(test)]
impl NativePluginLiveHost {
    fn registration_replay_context_build_counts(&self) -> RegistrationReplayContextBuildCounts {
        RegistrationReplayContextBuildCounts {
            registration_manifest_parses: self
                .registration_replay_context_build_counters
                .registration_manifest_parses
                .load(Ordering::Relaxed),
            registration_system_preparations: self
                .registration_replay_context_build_counters
                .registration_system_preparations
                .load(Ordering::Relaxed),
            package_manifest_snapshots: self
                .registration_replay_context_build_counters
                .package_manifest_snapshots
                .load(Ordering::Relaxed),
            binding_snapshots: self
                .registration_replay_context_build_counters
                .binding_snapshots
                .load(Ordering::Relaxed),
            method_lookup_builds: self
                .registration_replay_context_build_counters
                .method_lookup_builds
                .load(Ordering::Relaxed),
            bridge_call_scope_builds: self
                .registration_replay_context_build_counters
                .bridge_call_scope_builds
                .load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
#[path = "native_plugin_live_host/tests.rs"]
mod tests;
