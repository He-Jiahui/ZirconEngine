use crate::core::framework::scene::EntityId;

/// 可绘制的虚拟几何连续区间；ordinal/span 与同一实例的 cluster 排序合同绑定。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryDrawSegment {
    pub entity: EntityId,
    pub stable_instance_key: u64,
    pub cluster_id: u32,
    pub page_id: u32,
    pub cluster_ordinal: u32,
    pub cluster_span_count: u32,
    pub cluster_count: u32,
    pub lineage_depth: u32,
    pub lod_level: u8,
}
