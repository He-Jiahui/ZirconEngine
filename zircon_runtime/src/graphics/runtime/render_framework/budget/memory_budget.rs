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
#[path = "tests/memory_budget.rs"]
mod tests;
