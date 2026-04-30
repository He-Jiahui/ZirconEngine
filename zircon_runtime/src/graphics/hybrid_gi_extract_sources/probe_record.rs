use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics) struct HybridGiExtractProbeRecord {
    pub(in crate::graphics) entity: EntityId,
    pub(in crate::graphics) probe_id: u32,
    pub(in crate::graphics) position: Vec3,
    pub(in crate::graphics) radius: Real,
    pub(in crate::graphics) parent_probe_id: Option<u32>,
    pub(in crate::graphics) resident: bool,
    pub(in crate::graphics) ray_budget: u32,
}
