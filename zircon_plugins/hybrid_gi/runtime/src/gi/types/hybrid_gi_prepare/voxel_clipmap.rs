use zircon_runtime::core::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
/// 一层世界空间体素网格；half_extent 为中心到边界的距离，非正值不参与格子映射。
pub struct HybridGiPrepareVoxelClipmap {
    pub clipmap_id: u32,
    pub center: Vec3,
    pub half_extent: f32,
}
