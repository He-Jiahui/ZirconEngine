use zircon_framework::scene::EntityId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryCluster {
    pub entity: EntityId,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub cluster_ordinal: u32,
    pub cluster_count: u32,
    pub resident: bool,
}

