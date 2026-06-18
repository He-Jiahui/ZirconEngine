#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceNodeVisit {
    pub node_id: u32,
    pub depth: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub is_leaf: bool,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryCpuReferenceLeafCluster {
    pub node_id: u32,
    pub cluster_ordinal: u32,
    pub cluster_id: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub loaded: bool,
    pub parent_cluster_id: Option<u32>,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub screen_space_error: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceSelectedCluster {
    pub node_id: u32,
    pub cluster_ordinal: u32,
    pub cluster_id: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub loaded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferencePageClusterMapEntry {
    pub page_id: u32,
    pub cluster_ids: Vec<u32>,
}

/// Neutral debug view of the cooked VG page graph; plugin residency code owns any mutable streaming state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferencePageDependencyEntry {
    pub page_id: u32,
    pub parent_page_id: Option<u32>,
    pub child_page_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceDepthClusterMapEntry {
    pub depth: u32,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryCpuReferenceMipClusterMapEntry {
    pub mip_level: u8,
    pub cluster_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryCpuReferenceInstance {
    pub instance_index: u32,
    pub entity: u64,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
    pub visited_nodes: Vec<RenderVirtualGeometryCpuReferenceNodeVisit>,
    pub leaf_clusters: Vec<RenderVirtualGeometryCpuReferenceLeafCluster>,
    pub loaded_leaf_clusters: Vec<RenderVirtualGeometryCpuReferenceLeafCluster>,
    pub mip_accepted_clusters: Vec<RenderVirtualGeometryCpuReferenceLeafCluster>,
    pub selected_clusters: Vec<RenderVirtualGeometryCpuReferenceSelectedCluster>,
    pub page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub loaded_page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub mip_accepted_page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub page_dependencies: Vec<RenderVirtualGeometryCpuReferencePageDependencyEntry>,
    pub loaded_mip_cluster_map: Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry>,
    pub selected_page_cluster_map: Vec<RenderVirtualGeometryCpuReferencePageClusterMapEntry>,
    pub depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub loaded_depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub mip_accepted_depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub selected_depth_cluster_map: Vec<RenderVirtualGeometryCpuReferenceDepthClusterMapEntry>,
    pub mip_cluster_map: Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry>,
    pub selected_mip_cluster_map: Vec<RenderVirtualGeometryCpuReferenceMipClusterMapEntry>,
}
