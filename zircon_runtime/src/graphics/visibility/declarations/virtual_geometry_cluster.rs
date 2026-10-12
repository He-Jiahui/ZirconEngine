use crate::core::framework::scene::EntityId;

/// 主视图选中的虚拟几何 cluster；身份、LOD 与页驻留状态传给绘制段规划。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryCluster {
    pub entity: EntityId,
    pub stable_instance_key: u64,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub cluster_ordinal: u32,
    pub cluster_count: u32,
    pub resident: bool,
}
