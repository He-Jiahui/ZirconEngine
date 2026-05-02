use zircon_runtime::core::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub struct HybridGiPrepareVoxelClipmap {
    pub clipmap_id: u32,
    pub center: Vec3,
    pub half_extent: f32,
}
