use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use libloading::Library;

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

pub(in crate::plugin::native_plugin_loader) struct NativePluginStableLibrary {
    pub(super) library: Library,
    callback_activity: AtomicUsize,
    diagnostics_enabled: AtomicBool,
    diagnostic_shards:
        [NativePluginCallbackDiagnosticShard; NATIVE_CALLBACK_DIAGNOSTIC_SHARD_COUNT],
}

impl NativePluginStableLibrary {
    pub(super) fn new(library: Library) -> Arc<Self> {
        Arc::new(Self {
            library,
            callback_activity: AtomicUsize::new(0),
            diagnostics_enabled: AtomicBool::new(true),
            diagnostic_shards: std::array::from_fn(|_| {
                NativePluginCallbackDiagnosticShard::default()
            }),
        })
    }

    pub(super) fn acquire_callback(
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

    pub(super) fn begin_lifecycle_transition(
        &self,
    ) -> Result<(), NativePluginLifecycleTransitionError> {
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

    pub(super) fn cancel_lifecycle_transition(&self) {
        let result = self.callback_activity.compare_exchange(
            NATIVE_CALLBACK_TRANSITION_BIT,
            0,
            Ordering::Release,
            Ordering::Relaxed,
        );
        debug_assert_eq!(result, Ok(NATIVE_CALLBACK_TRANSITION_BIT));
    }

    pub(super) fn diagnostics(&self) -> NativePluginCallbackDiagnostics {
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

    pub(super) fn set_diagnostics_enabled(&self, enabled: bool) {
        self.diagnostics_enabled.store(enabled, Ordering::Release);
    }

    pub(super) fn begin_callback_measurement(&self) -> Option<Instant> {
        self.diagnostics_enabled
            .load(Ordering::Relaxed)
            .then(Instant::now)
    }

    pub(super) fn complete_callback_measurement(&self, started_at: Option<Instant>) {
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

pub(in crate::plugin::native_plugin_loader) struct NativePluginCallbackLease {
    owner: Arc<NativePluginStableLibrary>,
}

impl NativePluginCallbackLease {
    pub(in crate::plugin::native_plugin_loader) fn begin_callback_measurement(
        &self,
    ) -> Option<Instant> {
        self.owner.begin_callback_measurement()
    }

    pub(in crate::plugin::native_plugin_loader) fn complete_callback_measurement(
        &self,
        started_at: Option<Instant>,
    ) {
        self.owner.complete_callback_measurement(started_at);
    }
}

/// Passive ownership for a loaded dynamic-library generation. Retaining this owner keeps native
/// function pointers valid without counting as an executing callback or blocking a lifecycle
/// transition. A callback lease is acquired only immediately before foreign code is invoked.
#[derive(Clone)]
pub(in crate::plugin::native_plugin_loader) struct NativePluginLibraryGenerationOwner {
    owner: Arc<NativePluginStableLibrary>,
}

impl NativePluginLibraryGenerationOwner {
    pub(super) fn new(owner: Arc<NativePluginStableLibrary>) -> Self {
        Self { owner }
    }

    pub(in crate::plugin::native_plugin_loader) fn acquire_callback(
        &self,
    ) -> Result<NativePluginCallbackLease, NativePluginCallbackLeaseError> {
        self.owner.acquire_callback()
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
pub(in crate::plugin::native_plugin_loader) enum NativePluginCallbackLeaseError {
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
pub(in crate::plugin::native_plugin_loader) enum NativePluginLifecycleTransitionError {
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

const fn callback_transition_active(activity: usize) -> bool {
    activity & NATIVE_CALLBACK_TRANSITION_BIT != 0
}

const fn callback_count(activity: usize) -> usize {
    activity & NATIVE_CALLBACK_COUNT_MASK
}

fn duration_ns(duration: std::time::Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}
