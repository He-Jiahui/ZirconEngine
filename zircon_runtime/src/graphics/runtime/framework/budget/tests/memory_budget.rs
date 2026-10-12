use crate::core::framework::render::RenderFrameProfile;

use super::{is_memory_over_budget, memory_budget_warning_count, GpuMemoryBudget};

#[test]
fn render_perf_memory_budget_counts_each_exceeded_pool_once() {
    let budget = GpuMemoryBudget::new(100, 200, 300).with_persistent_texture_bytes(400);
    let profile = RenderFrameProfile {
        transient_texture_peak_bytes: 101,
        transient_buffer_peak_bytes: 200,
        staging_total_bytes: 301,
        persistent_texture_resident_bytes: 401,
        ..RenderFrameProfile::default()
    };

    assert_eq!(memory_budget_warning_count(&profile, budget), 3);
    assert!(is_memory_over_budget(&profile, budget));
}
