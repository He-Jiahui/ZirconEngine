use std::sync::atomic::{AtomicU64, Ordering};

static HOST_CALL_COUNT: AtomicU64 = AtomicU64::new(0);
static HOST_ARGUMENT_FRAME_ALLOCS: AtomicU64 = AtomicU64::new(0);
static HOST_ARGUMENT_DEEP_CLONE_BYTES: AtomicU64 = AtomicU64::new(0);
static GUEST_STRING_COPY_BYTES: AtomicU64 = AtomicU64::new(0);
static GUEST_BYTE_COPY_BYTES: AtomicU64 = AtomicU64::new(0);
static SCRIPT_CONTEXT_LEVEL_CLONES: AtomicU64 = AtomicU64::new(0);
static SCRIPT_CONTEXT_WEAK_HANDLES: AtomicU64 = AtomicU64::new(0);
static WORLD_SCOPE_ENTRIES: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScriptHostHotPathMetricsSnapshot {
    pub host_call_count: u64,
    pub host_argument_frame_allocs: u64,
    pub host_argument_deep_clone_bytes: u64,
    pub guest_string_copy_bytes: u64,
    pub guest_byte_copy_bytes: u64,
    pub script_context_level_clones: u64,
    pub script_context_weak_handles: u64,
    pub world_scope_entries: u64,
}

/// Allocation-free counters for script host-call ownership boundaries.
///
/// The counters are cumulative and intentionally avoid dynamic labels so they
/// can be sampled by runtime diagnostics without adding work to a guest call.
pub struct ScriptHostHotPathMetrics;

impl ScriptHostHotPathMetrics {
    pub fn snapshot() -> ScriptHostHotPathMetricsSnapshot {
        ScriptHostHotPathMetricsSnapshot {
            host_call_count: HOST_CALL_COUNT.load(Ordering::Relaxed),
            host_argument_frame_allocs: HOST_ARGUMENT_FRAME_ALLOCS.load(Ordering::Relaxed),
            host_argument_deep_clone_bytes: HOST_ARGUMENT_DEEP_CLONE_BYTES.load(Ordering::Relaxed),
            guest_string_copy_bytes: GUEST_STRING_COPY_BYTES.load(Ordering::Relaxed),
            guest_byte_copy_bytes: GUEST_BYTE_COPY_BYTES.load(Ordering::Relaxed),
            script_context_level_clones: SCRIPT_CONTEXT_LEVEL_CLONES.load(Ordering::Relaxed),
            script_context_weak_handles: SCRIPT_CONTEXT_WEAK_HANDLES.load(Ordering::Relaxed),
            world_scope_entries: WORLD_SCOPE_ENTRIES.load(Ordering::Relaxed),
        }
    }

    pub fn record_host_call() {
        HOST_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_guest_string_copy(byte_count: usize) {
        GUEST_STRING_COPY_BYTES.fetch_add(byte_count as u64, Ordering::Relaxed);
    }

    pub fn record_guest_byte_copy(byte_count: usize) {
        GUEST_BYTE_COPY_BYTES.fetch_add(byte_count as u64, Ordering::Relaxed);
    }

    pub fn record_script_context_level_clone() {
        SCRIPT_CONTEXT_LEVEL_CLONES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_script_context_weak_handle() {
        SCRIPT_CONTEXT_WEAK_HANDLES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_world_scope_entry() {
        WORLD_SCOPE_ENTRIES.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptHostHotPathMetrics;

    #[test]
    fn runtime13_context_construction_counters_increase_without_resetting_global_metrics() {
        let before = ScriptHostHotPathMetrics::snapshot();

        ScriptHostHotPathMetrics::record_script_context_weak_handle();
        ScriptHostHotPathMetrics::record_script_context_level_clone();

        let after = ScriptHostHotPathMetrics::snapshot();
        assert!(
            after.script_context_weak_handles
                >= before.script_context_weak_handles.saturating_add(1),
            "weak-handle construction counter should increase"
        );
        assert!(
            after.script_context_level_clones
                >= before.script_context_level_clones.saturating_add(1),
            "level-clone construction counter should increase"
        );
    }
}
