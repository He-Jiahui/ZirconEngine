use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics) struct HybridGiExtractTraceRegionRecord {
    pub(in crate::graphics) entity: EntityId,
    pub(in crate::graphics) region_id: u32,
    pub(in crate::graphics) bounds_center: Vec3,
    pub(in crate::graphics) bounds_radius: Real,
    pub(in crate::graphics) screen_coverage: Real,
    pub(in crate::graphics) rt_lighting_rgb: [u8; 3],
}
