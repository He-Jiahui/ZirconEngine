#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard};
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;

#[cfg(test)]
use super::NativeBridgeMethodBinding;
use super::{LoadedNativePlugin, NativePluginCallbackDiagnostics, NativePluginLoader};
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

use bridge_methods::ValidatedRuntimeBridgeMethodBindings;
use keys::NativePluginLiveRegistry;
use registration_replay::{
    NativePluginRegistrationReplayBridgeContext, NativePluginRegistrationReplayGeneration,
};

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

#[cfg(test)]
const NATIVE_PLUGIN_LIVE_HOST_TEST_GATE_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(test)]
struct NativePluginLiveHostTestGateTrigger {
    reached: mpsc::Sender<()>,
    resume: Mutex<mpsc::Receiver<()>>,
}

#[cfg(test)]
struct NativePluginLiveHostTestGate {
    reached: mpsc::Receiver<()>,
    resume: Option<mpsc::Sender<()>>,
}

#[cfg(test)]
impl NativePluginLiveHostTestGate {
    fn wait_until_reached(&self, description: &str) {
        self.reached
            .recv_timeout(NATIVE_PLUGIN_LIVE_HOST_TEST_GATE_TIMEOUT)
            .unwrap_or_else(|error| panic!("{description} did not reach its test gate: {error}"));
    }

    fn resume(&mut self) {
        if let Some(resume) = self.resume.take() {
            let _ = resume.send(());
        }
    }
}

#[cfg(test)]
impl Drop for NativePluginLiveHostTestGate {
    fn drop(&mut self) {
        self.resume();
    }
}

#[cfg(test)]
#[derive(Default)]
struct NativePluginLiveHostTestGateHook {
    installed: Mutex<Option<Arc<NativePluginLiveHostTestGateTrigger>>>,
}

#[cfg(test)]
impl std::fmt::Debug for NativePluginLiveHostTestGateHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginLiveHostTestGateHook")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
impl NativePluginLiveHostTestGateHook {
    fn install(&self) -> NativePluginLiveHostTestGate {
        let (reached_tx, reached_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let trigger = Arc::new(NativePluginLiveHostTestGateTrigger {
            reached: reached_tx,
            resume: Mutex::new(resume_rx),
        });
        let mut installed = self
            .installed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(installed.is_none(), "test gate is already installed");
        *installed = Some(trigger);
        NativePluginLiveHostTestGate {
            reached: reached_rx,
            resume: Some(resume_tx),
        }
    }

    fn pause_if_installed(&self) {
        let trigger = self
            .installed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        let Some(trigger) = trigger else {
            return;
        };
        if trigger.reached.send(()).is_err() {
            return;
        }
        let _ = trigger
            .resume
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .recv_timeout(NATIVE_PLUGIN_LIVE_HOST_TEST_GATE_TIMEOUT);
    }
}

#[derive(Debug, Default)]
pub struct NativePluginLiveHost {
    loader: NativePluginLoader,
    loaded: ObservedLoadedNativePlugins,
    runtime_bridge_method_bindings:
        Mutex<NativePluginLiveRegistry<ValidatedRuntimeBridgeMethodBindings>>,
    // Bridge helpers and registration replay share one immutable slot/scope generation. The
    // registration layer adds parsed manifest data without rebuilding that lower projection.
    runtime_bridge_generations:
        Mutex<NativePluginLiveRegistry<Arc<NativePluginRegistrationReplayBridgeContext>>>,
    runtime_bridge_generation_build_lock: Mutex<()>,
    runtime_registration_replay_generations:
        Mutex<NativePluginLiveRegistry<Arc<NativePluginRegistrationReplayGeneration>>>,
    runtime_registration_replay_generation_build_lock: Mutex<()>,
    // The revision prevents a late builder from publishing an older load or binding generation.
    runtime_registration_replay_generation_revisions: Mutex<NativePluginLiveRegistry<u64>>,
    #[cfg(test)]
    registration_replay_context_build_counters: RegistrationReplayContextBuildCounters,
    #[cfg(test)]
    registration_replay_source_test_hook: NativePluginLiveHostTestGateHook,
    #[cfg(test)]
    registration_replay_before_cache_test_hook: NativePluginLiveHostTestGateHook,
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
    #[cfg(test)]
    lock_attempts: AtomicU64,
    #[cfg(test)]
    after_lock_test_hook: NativePluginLiveHostTestGateHook,
    lock_acquisitions: AtomicU64,
    total_lock_wait_ns: AtomicU64,
    max_lock_wait_ns: AtomicU64,
}

impl ObservedLoadedNativePlugins {
    fn lock(
        &self,
    ) -> std::sync::LockResult<MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>> {
        #[cfg(test)]
        self.lock_attempts.fetch_add(1, Ordering::Release);
        let wait_started = Instant::now();
        let result = self.entries.lock();
        #[cfg(test)]
        if result.is_ok() {
            self.after_lock_test_hook.pause_if_installed();
        }
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

    #[cfg(test)]
    fn is_unlocked(&self) -> bool {
        self.entries.try_lock().is_ok()
    }

    #[cfg(test)]
    fn lock_attempts(&self) -> u64 {
        self.lock_attempts.load(Ordering::Acquire)
    }

    #[cfg(test)]
    fn install_after_lock_test_gate(&self) -> NativePluginLiveHostTestGate {
        self.after_lock_test_hook.install()
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
