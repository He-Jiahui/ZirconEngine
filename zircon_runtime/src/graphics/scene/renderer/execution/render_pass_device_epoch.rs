use crate::rhi::RenderDeviceProfile;

/// Device generation that materialized one render-graph execution packet.
///
/// The raw identity stays opaque to render passes. They can compare the complete epoch but cannot
/// accidentally use one scalar without the other or reconstruct native device ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderPassDeviceEpoch {
    device_id: u64,
    generation: u64,
}

impl RenderPassDeviceEpoch {
    pub(in crate::graphics::scene::scene_renderer) const fn from_profile(
        profile: &RenderDeviceProfile,
    ) -> Self {
        Self::new(profile.device_id().raw(), profile.generation().raw())
    }

    pub(in crate::graphics::scene::scene_renderer) const fn new(
        device_id: u64,
        generation: u64,
    ) -> Self {
        Self {
            device_id,
            generation,
        }
    }

    pub(crate) const fn raw_parts(self) -> (u64, u64) {
        (self.device_id, self.generation)
    }
}
