use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HybridGiExtractProbeRecord {
    pub entity: EntityId,
    pub probe_id: u32,
    pub position: Vec3,
    pub radius: Real,
    pub parent_probe_id: Option<u32>,
    pub resident: bool,
    pub ray_budget: u32,
}
