use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMeshLodSelection {
    pub level_index: u32,
    pub min_distance: Real,
}

impl RenderMeshLodSelection {
    pub fn new(level_index: u32, min_distance: Real) -> Self {
        Self {
            level_index,
            min_distance,
        }
    }
}
