use crate::core::framework::render::RenderFrameProfile;
use zr_rhi::GpuMemoryBudget;

pub(in crate::graphics::runtime::render_framework) fn memory_budget_warning_count(
    profile: &RenderFrameProfile,
    budget: GpuMemoryBudget,
) -> u32 {
    u32::from(profile.transient_texture_peak_bytes > budget.transient_texture_bytes())
        + u32::from(profile.transient_buffer_peak_bytes > budget.transient_buffer_bytes())
        + u32::from(profile.staging_total_bytes > budget.staging_bytes())
        + u32::from(profile.persistent_texture_resident_bytes > budget.persistent_texture_bytes())
}

pub(in crate::graphics::runtime::render_framework) fn is_memory_over_budget(
    profile: &RenderFrameProfile,
    budget: GpuMemoryBudget,
) -> bool {
    memory_budget_warning_count(profile, budget) != 0
}

#[cfg(test)]
mod tests {
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
}
