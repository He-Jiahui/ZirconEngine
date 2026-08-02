use crate::core::framework::render::RenderFrameProfile;

const REFERENCE_TRANSIENT_TEXTURE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;
const REFERENCE_TRANSIENT_BUFFER_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
const REFERENCE_STAGING_BUDGET_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::runtime::render_framework) struct RenderMemoryBudget {
    transient_texture_bytes: u64,
    transient_buffer_bytes: u64,
    staging_bytes: u64,
}

impl RenderMemoryBudget {
    pub(in crate::graphics::runtime::render_framework) const fn new(
        transient_texture_bytes: u64,
        transient_buffer_bytes: u64,
        staging_bytes: u64,
    ) -> Self {
        Self {
            transient_texture_bytes,
            transient_buffer_bytes,
            staging_bytes,
        }
    }

    pub(in crate::graphics::runtime::render_framework) const fn reference_1080p_mid() -> Self {
        Self::new(
            REFERENCE_TRANSIENT_TEXTURE_BUDGET_BYTES,
            REFERENCE_TRANSIENT_BUFFER_BUDGET_BYTES,
            REFERENCE_STAGING_BUDGET_BYTES,
        )
    }

    pub(in crate::graphics::runtime::render_framework) fn warning_count(
        &self,
        profile: &RenderFrameProfile,
    ) -> u32 {
        u32::from(profile.transient_texture_peak_bytes > self.transient_texture_bytes)
            + u32::from(profile.transient_buffer_peak_bytes > self.transient_buffer_bytes)
            + u32::from(profile.staging_total_bytes > self.staging_bytes)
    }

    pub(in crate::graphics::runtime::render_framework) fn is_over_budget(
        &self,
        profile: &RenderFrameProfile,
    ) -> bool {
        self.warning_count(profile) != 0
    }
}

impl Default for RenderMemoryBudget {
    fn default() -> Self {
        Self::reference_1080p_mid()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderFrameProfile;

    use super::RenderMemoryBudget;

    #[test]
    fn render_perf_memory_budget_counts_each_exceeded_pool_once() {
        let budget = RenderMemoryBudget::new(100, 200, 300);
        let profile = RenderFrameProfile {
            transient_texture_peak_bytes: 101,
            transient_buffer_peak_bytes: 200,
            staging_total_bytes: 301,
            ..RenderFrameProfile::default()
        };

        assert_eq!(budget.warning_count(&profile), 2);
        assert!(budget.is_over_budget(&profile));
    }
}
