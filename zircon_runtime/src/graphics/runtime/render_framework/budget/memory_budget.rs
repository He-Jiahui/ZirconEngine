use crate::core::framework::render::RenderFrameProfile;

const REFERENCE_TRANSIENT_TEXTURE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;
const REFERENCE_TRANSIENT_BUFFER_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
const REFERENCE_STAGING_BUDGET_BYTES: u64 = 64 * 1024 * 1024;
const REFERENCE_PERSISTENT_TEXTURE_BUDGET_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::runtime::render_framework) struct RenderMemoryBudget {
    transient_texture_bytes: u64,
    transient_buffer_bytes: u64,
    staging_bytes: u64,
    persistent_texture_bytes: u64,
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
            persistent_texture_bytes: REFERENCE_PERSISTENT_TEXTURE_BUDGET_BYTES,
        }
    }

    pub(in crate::graphics::runtime::render_framework) const fn with_persistent_texture_bytes(
        mut self,
        persistent_texture_bytes: u64,
    ) -> Self {
        self.persistent_texture_bytes = persistent_texture_bytes;
        self
    }

    pub(in crate::graphics::runtime::render_framework) const fn reference_1080p_mid() -> Self {
        Self::new(
            REFERENCE_TRANSIENT_TEXTURE_BUDGET_BYTES,
            REFERENCE_TRANSIENT_BUFFER_BUDGET_BYTES,
            REFERENCE_STAGING_BUDGET_BYTES,
        )
        .with_persistent_texture_bytes(REFERENCE_PERSISTENT_TEXTURE_BUDGET_BYTES)
    }

    pub(in crate::graphics::runtime::render_framework) const fn persistent_texture_bytes(
        &self,
    ) -> u64 {
        self.persistent_texture_bytes
    }

    pub(in crate::graphics::runtime::render_framework) fn warning_count(
        &self,
        profile: &RenderFrameProfile,
    ) -> u32 {
        u32::from(profile.transient_texture_peak_bytes > self.transient_texture_bytes)
            + u32::from(profile.transient_buffer_peak_bytes > self.transient_buffer_bytes)
            + u32::from(profile.staging_total_bytes > self.staging_bytes)
            + u32::from(profile.persistent_texture_resident_bytes > self.persistent_texture_bytes)
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
        let budget = RenderMemoryBudget::new(100, 200, 300).with_persistent_texture_bytes(400);
        let profile = RenderFrameProfile {
            transient_texture_peak_bytes: 101,
            transient_buffer_peak_bytes: 200,
            staging_total_bytes: 301,
            persistent_texture_resident_bytes: 401,
            ..RenderFrameProfile::default()
        };

        assert_eq!(budget.warning_count(&profile), 3);
        assert!(budget.is_over_budget(&profile));
    }
}
