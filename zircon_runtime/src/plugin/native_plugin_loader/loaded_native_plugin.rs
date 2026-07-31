use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use libloading::Library;

use crate::asset::{
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
};

use super::behavior_calls::{NativePluginBehavior, NativePluginBehaviorCallbacks};

use super::{
    NativePluginBehaviorCallReport, NativePluginBehaviorHealth,
    NativePluginBehaviorValidationReport, NativePluginDescriptor, NativePluginEntryReport,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
};

impl NativeAssetImportCommandHost for LoadedNativePlugin {
    fn command_host_id(&self) -> &str {
        &self.plugin_id
    }

    fn invoke_asset_import_command(
        &self,
        command: &str,
        payload: &[u8],
    ) -> NativeAssetImportCommandReport {
        let report = LoadedNativePlugin::invoke_runtime_command(self, command, payload);
        let status = match report.status_code {
            super::ZIRCON_NATIVE_PLUGIN_STATUS_OK => NativeAssetImportCommandStatus::Ok,
            super::ZIRCON_NATIVE_PLUGIN_STATUS_ERROR => NativeAssetImportCommandStatus::Error,
            super::ZIRCON_NATIVE_PLUGIN_STATUS_DENIED => NativeAssetImportCommandStatus::Denied,
            super::ZIRCON_NATIVE_PLUGIN_STATUS_PANIC => NativeAssetImportCommandStatus::Panic,
            status => NativeAssetImportCommandStatus::Unknown(status),
        };
        NativeAssetImportCommandReport {
            status,
            diagnostics: report.diagnostics,
            payload: report.payload,
        }
    }
}

#[derive(Clone)]
pub struct LoadedNativePlugin {
    pub plugin_id: String,
    pub library_path: PathBuf,
    pub descriptor: Option<NativePluginDescriptor>,
    pub runtime_entry_report: Option<NativePluginEntryReport>,
    pub editor_entry_report: Option<NativePluginEntryReport>,
    pub(super) library: Arc<NativePluginStableLibrary>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativePluginCallbackDiagnostics {
    pub active_callbacks: usize,
    pub completed_callbacks: u64,
    pub total_callback_duration_ns: u64,
    pub max_callback_duration_ns: u64,
    /// Retained for report compatibility. Stable callback admission no longer acquires a mutex.
    pub lifecycle_lock_wait_ns: u64,
    pub lifecycle_transition_active: bool,
    pub callback_state_mutex_acquisitions: u64,
    pub diagnostics_enabled: bool,
    pub diagnostic_shard_count: usize,
}

// Each Arc owner is one load generation. Its highest state bit closes admission while the
// remaining bits count callbacks that still pin that generation's dynamic library.
const NATIVE_CALLBACK_TRANSITION_BIT: usize = 1 << (usize::BITS - 1);
const NATIVE_CALLBACK_COUNT_MASK: usize = NATIVE_CALLBACK_TRANSITION_BIT - 1;
const NATIVE_CALLBACK_DIAGNOSTIC_SHARD_COUNT: usize = 64;

static NEXT_NATIVE_CALLBACK_DIAGNOSTIC_SHARD: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static NATIVE_CALLBACK_DIAGNOSTIC_SHARD_INDEX: usize =
        NEXT_NATIVE_CALLBACK_DIAGNOSTIC_SHARD.fetch_add(1, Ordering::Relaxed)
            % NATIVE_CALLBACK_DIAGNOSTIC_SHARD_COUNT;
}

#[repr(align(64))]
#[derive(Debug, Default)]
struct NativePluginCallbackDiagnosticShard {
    completed_callbacks: AtomicU64,
    total_callback_duration_ns: AtomicU64,
    max_callback_duration_ns: AtomicU64,
}

impl NativePluginCallbackDiagnosticShard {
    fn record(&self, elapsed_ns: u64) {
        self.completed_callbacks.fetch_add(1, Ordering::Relaxed);
        self.total_callback_duration_ns
            .fetch_add(elapsed_ns, Ordering::Relaxed);
        self.max_callback_duration_ns
            .fetch_max(elapsed_ns, Ordering::Relaxed);
    }
}

pub(super) struct NativePluginStableLibrary {
    library: Library,
    callback_activity: AtomicUsize,
    diagnostics_enabled: AtomicBool,
    diagnostic_shards:
        [NativePluginCallbackDiagnosticShard; NATIVE_CALLBACK_DIAGNOSTIC_SHARD_COUNT],
}

impl NativePluginStableLibrary {
    fn new(library: Library) -> Arc<Self> {
        Arc::new(Self {
            library,
            callback_activity: AtomicUsize::new(0),
            diagnostics_enabled: AtomicBool::new(true),
            diagnostic_shards: std::array::from_fn(|_| {
                NativePluginCallbackDiagnosticShard::default()
            }),
        })
    }

    fn acquire_callback(
        self: &Arc<Self>,
    ) -> Result<NativePluginCallbackLease, NativePluginCallbackLeaseError> {
        let mut observed = self.callback_activity.load(Ordering::Acquire);
        loop {
            if callback_transition_active(observed) {
                return Err(NativePluginCallbackLeaseError::LifecycleTransitionActive);
            }
            if callback_count(observed) == NATIVE_CALLBACK_COUNT_MASK {
                return Err(NativePluginCallbackLeaseError::ActiveCallbackLimitReached);
            }
            match self.callback_activity.compare_exchange_weak(
                observed,
                observed + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Ok(NativePluginCallbackLease {
                        owner: self.clone(),
                    });
                }
                Err(current) => observed = current,
            }
        }
    }

    fn begin_lifecycle_transition(&self) -> Result<(), NativePluginLifecycleTransitionError> {
        match self.callback_activity.compare_exchange(
            0,
            NATIVE_CALLBACK_TRANSITION_BIT,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(()),
            Err(observed) if callback_transition_active(observed) => {
                Err(NativePluginLifecycleTransitionError::AlreadyTransitioning)
            }
            Err(observed) => Err(NativePluginLifecycleTransitionError::ActiveCallbacks {
                count: callback_count(observed),
            }),
        }
    }

    fn cancel_lifecycle_transition(&self) {
        let result = self.callback_activity.compare_exchange(
            NATIVE_CALLBACK_TRANSITION_BIT,
            0,
            Ordering::Release,
            Ordering::Relaxed,
        );
        debug_assert_eq!(result, Ok(NATIVE_CALLBACK_TRANSITION_BIT));
    }

    fn diagnostics(&self) -> NativePluginCallbackDiagnostics {
        let activity = self.callback_activity.load(Ordering::Acquire);
        let mut completed_callbacks = 0_u64;
        let mut total_callback_duration_ns = 0_u64;
        let mut max_callback_duration_ns = 0_u64;
        for shard in &self.diagnostic_shards {
            completed_callbacks = completed_callbacks
                .saturating_add(shard.completed_callbacks.load(Ordering::Relaxed));
            total_callback_duration_ns = total_callback_duration_ns
                .saturating_add(shard.total_callback_duration_ns.load(Ordering::Relaxed));
            max_callback_duration_ns = max_callback_duration_ns
                .max(shard.max_callback_duration_ns.load(Ordering::Relaxed));
        }
        NativePluginCallbackDiagnostics {
            active_callbacks: callback_count(activity),
            completed_callbacks,
            total_callback_duration_ns,
            max_callback_duration_ns,
            lifecycle_lock_wait_ns: 0,
            lifecycle_transition_active: callback_transition_active(activity),
            callback_state_mutex_acquisitions: 0,
            diagnostics_enabled: self.diagnostics_enabled.load(Ordering::Relaxed),
            diagnostic_shard_count: NATIVE_CALLBACK_DIAGNOSTIC_SHARD_COUNT,
        }
    }

    fn set_diagnostics_enabled(&self, enabled: bool) {
        self.diagnostics_enabled.store(enabled, Ordering::Release);
    }

    fn begin_callback_measurement(&self) -> Option<Instant> {
        self.diagnostics_enabled
            .load(Ordering::Relaxed)
            .then(Instant::now)
    }

    fn complete_callback_measurement(&self, started_at: Option<Instant>) {
        let Some(started_at) = started_at else {
            return;
        };
        let elapsed_ns = duration_ns(started_at.elapsed());
        NATIVE_CALLBACK_DIAGNOSTIC_SHARD_INDEX.with(|shard_index| {
            self.diagnostic_shards[*shard_index].record(elapsed_ns);
        });
    }
}

impl std::fmt::Debug for NativePluginStableLibrary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginStableLibrary")
            .field("diagnostics", &self.diagnostics())
            .finish_non_exhaustive()
    }
}

pub(super) struct NativePluginCallbackLease {
    owner: Arc<NativePluginStableLibrary>,
}

impl NativePluginCallbackLease {
    fn begin_callback_measurement(&self) -> Option<Instant> {
        self.owner.begin_callback_measurement()
    }

    fn complete_callback_measurement(&self, started_at: Option<Instant>) {
        self.owner.complete_callback_measurement(started_at);
    }
}

impl Drop for NativePluginCallbackLease {
    fn drop(&mut self) {
        let previous = self.owner.callback_activity.fetch_sub(1, Ordering::Release);
        debug_assert!(!callback_transition_active(previous));
        debug_assert!(callback_count(previous) > 0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativePluginCallbackLeaseError {
    LifecycleTransitionActive,
    ActiveCallbackLimitReached,
}

impl std::fmt::Display for NativePluginCallbackLeaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LifecycleTransitionActive => {
                formatter.write_str("native plugin lifecycle transition is active")
            }
            Self::ActiveCallbackLimitReached => {
                formatter.write_str("native plugin active callback limit was reached")
            }
        }
    }
}

impl std::error::Error for NativePluginCallbackLeaseError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativePluginLifecycleTransitionError {
    ActiveCallbacks { count: usize },
    AlreadyTransitioning,
}

impl std::fmt::Display for NativePluginLifecycleTransitionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActiveCallbacks { count } => write!(
                formatter,
                "native plugin lifecycle transition rejected because {count} active native callback(s) still hold the stable library owner"
            ),
            Self::AlreadyTransitioning => {
                formatter.write_str("native plugin lifecycle transition is already active")
            }
        }
    }
}

impl std::error::Error for NativePluginLifecycleTransitionError {}

pub(super) struct NativePluginBehaviorSnapshot {
    behavior: Option<NativePluginBehaviorCallbacks>,
    module_kind: &'static str,
    // The lease starts at snapshot freeze to block unload, while duration starts only when an
    // operation is invoked so broadcast queueing is not reported as callback execution time.
    _callback_lease: NativePluginCallbackLease,
}

impl NativePluginBehaviorSnapshot {
    pub(super) fn invoke_command(
        &self,
        name: &str,
        payload: &[u8],
    ) -> NativePluginBehaviorCallReport {
        self.invoke_measured(|| {
            self.behavior.as_ref().map_or_else(
                || missing_behavior_report(self.module_kind),
                |behavior| behavior.invoke_command(name, payload),
            )
        })
    }

    pub(super) fn save_state(&self) -> NativePluginBehaviorCallReport {
        self.invoke_measured(|| {
            self.behavior.as_ref().map_or_else(
                || missing_behavior_report(self.module_kind),
                |behavior| behavior.save_state(),
            )
        })
    }

    pub(super) fn restore_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        self.invoke_measured(|| {
            self.behavior.as_ref().map_or_else(
                || missing_behavior_report(self.module_kind),
                |behavior| behavior.restore_state(state),
            )
        })
    }

    pub(super) fn unload(&self) -> NativePluginBehaviorCallReport {
        self.invoke_measured(|| {
            self.behavior.as_ref().map_or_else(
                || missing_behavior_report(self.module_kind),
                |behavior| behavior.unload(),
            )
        })
    }

    fn invoke_measured(
        &self,
        callback: impl FnOnce() -> NativePluginBehaviorCallReport,
    ) -> NativePluginBehaviorCallReport {
        let started_at = self._callback_lease.begin_callback_measurement();
        let report = callback();
        self._callback_lease
            .complete_callback_measurement(started_at);
        report
    }
}

impl LoadedNativePlugin {
    pub(super) fn stable_library(library: Library) -> Arc<NativePluginStableLibrary> {
        NativePluginStableLibrary::new(library)
    }

    pub fn is_loaded(&self) -> bool {
        let _ = &self.library.library;
        true
    }

    pub fn callback_diagnostics(&self) -> NativePluginCallbackDiagnostics {
        self.library.diagnostics()
    }

    /// Enables or disables duration/count aggregation without changing callback admission.
    pub fn set_callback_diagnostics_enabled(&self, enabled: bool) {
        self.library.set_diagnostics_enabled(enabled);
    }

    pub(super) fn callback_owner_lease(
        &self,
    ) -> Result<NativePluginCallbackLease, NativePluginCallbackLeaseError> {
        self.library.acquire_callback()
    }

    fn callback_execution_lease(
        &self,
    ) -> Result<NativePluginCallbackLease, NativePluginCallbackLeaseError> {
        self.library.acquire_callback()
    }

    pub(super) fn runtime_behavior_snapshot(
        &self,
    ) -> Result<NativePluginBehaviorSnapshot, NativePluginCallbackLeaseError> {
        let callback_lease = self.callback_execution_lease()?;
        Ok(NativePluginBehaviorSnapshot {
            behavior: self
                .runtime_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref())
                .map(NativePluginBehavior::callback_snapshot),
            module_kind: "runtime",
            _callback_lease: callback_lease,
        })
    }

    fn editor_behavior_snapshot(
        &self,
    ) -> Result<NativePluginBehaviorSnapshot, NativePluginCallbackLeaseError> {
        let callback_lease = self.callback_execution_lease()?;
        Ok(NativePluginBehaviorSnapshot {
            behavior: self
                .editor_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref())
                .map(NativePluginBehavior::callback_snapshot),
            module_kind: "editor",
            _callback_lease: callback_lease,
        })
    }

    pub(super) fn begin_lifecycle_transition(
        &self,
    ) -> Result<(), NativePluginLifecycleTransitionError> {
        self.library.begin_lifecycle_transition()
    }

    pub(super) fn cancel_lifecycle_transition(&self) {
        self.library.cancel_lifecycle_transition();
    }

    pub fn runtime_behavior_is_stateless(&self) -> Option<bool> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .map(|behavior| behavior.is_stateless)
    }

    pub fn runtime_behavior_validation_report(
        &self,
    ) -> Option<&NativePluginBehaviorValidationReport> {
        self.runtime_entry_report
            .as_ref()
            .map(|report| &report.behavior_validation)
    }

    pub fn editor_behavior_validation_report(
        &self,
    ) -> Option<&NativePluginBehaviorValidationReport> {
        self.editor_entry_report
            .as_ref()
            .map(|report| &report.behavior_validation)
    }

    pub fn runtime_behavior_health(&self) -> Option<NativePluginBehaviorHealth> {
        self.runtime_behavior_validation_report()
            .map(|report| report.health)
    }

    pub fn editor_behavior_health(&self) -> Option<NativePluginBehaviorHealth> {
        self.editor_behavior_validation_report()
            .map(|report| report.health)
    }

    pub fn editor_behavior_is_stateless(&self) -> Option<bool> {
        self.editor_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .map(|behavior| behavior.is_stateless)
    }

    pub fn runtime_command_manifest(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.command_manifest.as_deref())
    }

    pub fn runtime_event_manifest(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.event_manifest.as_deref())
    }

    pub fn runtime_registration_manifest(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.registration_manifest.as_deref())
    }

    pub fn runtime_state_schema_version(&self) -> Option<u32> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .map(|behavior| behavior.state_schema_version)
    }

    pub fn runtime_command_manifest_schema(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.command_manifest_schema.as_deref())
    }

    pub fn runtime_event_manifest_schema(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.event_manifest_schema.as_deref())
    }

    pub fn runtime_registration_manifest_schema(&self) -> Option<&str> {
        self.runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref())
            .and_then(|behavior| behavior.registration_manifest_schema.as_deref())
    }

    pub fn invoke_runtime_command(
        &self,
        name: &str,
        payload: &[u8],
    ) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.invoke_command(name, payload),
        )
    }

    pub fn save_runtime_state(&self) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.save_state(),
        )
    }

    pub fn restore_runtime_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.restore_state(state),
        )
    }

    pub fn unload_runtime_behavior(&self) -> NativePluginBehaviorCallReport {
        self.runtime_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("runtime", error),
            |snapshot| snapshot.unload(),
        )
    }

    pub fn save_editor_state(&self) -> NativePluginBehaviorCallReport {
        self.editor_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("editor", error),
            |snapshot| snapshot.save_state(),
        )
    }

    pub fn unload_editor_behavior(&self) -> NativePluginBehaviorCallReport {
        self.editor_behavior_snapshot().map_or_else(
            |error| callback_rejected_report("editor", error),
            |snapshot| snapshot.unload(),
        )
    }

    pub(super) fn save_runtime_state_during_transition(&self) -> NativePluginBehaviorCallReport {
        let behavior = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref());
        behavior.map_or_else(
            || missing_behavior_report("runtime"),
            |behavior| self.invoke_lifecycle_callback(|| behavior.save_state()),
        )
    }

    pub(super) fn restore_runtime_state_during_transition(
        &self,
        state: &[u8],
    ) -> NativePluginBehaviorCallReport {
        let behavior = self
            .runtime_entry_report
            .as_ref()
            .and_then(|report| report.behavior.as_ref());
        behavior.map_or_else(
            || missing_behavior_report("runtime"),
            |behavior| self.invoke_lifecycle_callback(|| behavior.restore_state(state)),
        )
    }

    pub(super) fn unload_behavior_during_transition(
        &self,
        module_kind: crate::plugin::PluginModuleKind,
    ) -> NativePluginBehaviorCallReport {
        let behavior = match module_kind {
            crate::plugin::PluginModuleKind::Runtime => self
                .runtime_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref()),
            crate::plugin::PluginModuleKind::Editor => self
                .editor_entry_report
                .as_ref()
                .and_then(|report| report.behavior.as_ref()),
            crate::plugin::PluginModuleKind::Native | crate::plugin::PluginModuleKind::Vm => None,
        };
        behavior.map_or_else(
            || {
                missing_behavior_report(match module_kind {
                    crate::plugin::PluginModuleKind::Editor => "editor",
                    _ => "runtime",
                })
            },
            |behavior| self.invoke_lifecycle_callback(|| behavior.unload()),
        )
    }

    fn invoke_lifecycle_callback(
        &self,
        callback: impl FnOnce() -> NativePluginBehaviorCallReport,
    ) -> NativePluginBehaviorCallReport {
        let started_at = self.library.begin_callback_measurement();
        let report = callback();
        self.library.complete_callback_measurement(started_at);
        report
    }
}

const fn callback_transition_active(activity: usize) -> bool {
    activity & NATIVE_CALLBACK_TRANSITION_BIT != 0
}

const fn callback_count(activity: usize) -> usize {
    activity & NATIVE_CALLBACK_COUNT_MASK
}

fn callback_rejected_report(
    module_kind: &str,
    error: NativePluginCallbackLeaseError,
) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![format!(
            "native plugin {module_kind} behavior callback rejected: {error}"
        )],
        payload: None,
    }
}

fn duration_ns(duration: std::time::Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn missing_behavior_report(module_kind: &str) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![format!("native plugin {module_kind} behavior is missing")],
        payload: None,
    }
}

impl std::fmt::Debug for LoadedNativePlugin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoadedNativePlugin")
            .field("plugin_id", &self.plugin_id)
            .field("library_path", &self.library_path)
            .field("descriptor", &self.descriptor)
            .field("runtime_entry_report", &self.runtime_entry_report)
            .field("editor_entry_report", &self.editor_entry_report)
            .finish_non_exhaustive()
    }
}
