use crate::core::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HybridGiPrepareVoxelClipmap {
    pub(crate) clipmap_id: u32,
    pub(crate) center: Vec3,
    pub(crate) half_extent: f32,
}
