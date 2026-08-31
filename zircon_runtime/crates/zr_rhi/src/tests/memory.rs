use crate::{GpuMemoryBudget, GpuMemorySnapshot};

#[test]
fn reference_memory_budget_retains_the_1080p_mid_tier_limits() {
    let budget = GpuMemoryBudget::reference_1080p_mid();

    assert_eq!(budget.transient_texture_bytes(), 512 * 1024 * 1024);
    assert_eq!(budget.transient_buffer_bytes(), 256 * 1024 * 1024);
    assert_eq!(budget.staging_bytes(), 64 * 1024 * 1024);
    assert_eq!(budget.persistent_texture_bytes(), 1024 * 1024 * 1024);
    assert_eq!(budget.max_pending_uploads(), 16);
}

#[test]
fn gpu_memory_snapshot_keeps_active_retired_and_upload_bytes_disjoint() {
    let snapshot = GpuMemorySnapshot {
        active_buffer_bytes: 16,
        active_texture_bytes: 32,
        retired_buffer_bytes: 64,
        retired_texture_bytes: 128,
        pending_upload_bytes: 8,
        active_allocations: 2,
        retired_allocations: 3,
    };

    assert_eq!(snapshot.active_resource_bytes(), 48);
    assert_eq!(snapshot.retired_resource_bytes(), 192);
    assert_eq!(snapshot.reserved_resource_bytes(), 240);
    assert_eq!(snapshot.reserved_resource_allocations(), 5);
    assert_eq!(snapshot.pending_upload_bytes, 8);
}
