use crate::core::framework::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityHybridGiProbe {
    pub entity: EntityId,
    pub probe_id: u32,
    pub resident: bool,
    pub ray_budget: u32,
}
