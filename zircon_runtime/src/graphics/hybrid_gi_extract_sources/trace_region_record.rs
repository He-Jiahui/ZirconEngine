use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HybridGiExtractTraceRegionRecord {
    pub entity: EntityId,
    pub region_id: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: Real,
    pub screen_coverage: Real,
    pub rt_lighting_rgb: [u8; 3],
}
