#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryBvhVisualizationNode {
    pub node_id: u32,
    pub parent_node_id: Option<u32>,
    pub child_node_ids: Vec<u32>,
    pub depth: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub is_leaf: bool,
    pub cluster_ids: Vec<u32>,
    pub selected_cluster_ids: Vec<u32>,
    pub resident_cluster_ids: Vec<u32>,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub screen_space_error: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryBvhVisualizationInstance {
    pub instance_index: u32,
    pub entity: u64,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
    pub nodes: Vec<RenderVirtualGeometryBvhVisualizationNode>,
}
