//! 虚拟几何元数据随模型/网格派生产物传给资源流；页与 cluster 的索引语义由烹饪和驻留加载共同约束。

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VirtualGeometryHierarchyNodeAsset {
    pub node_id: u32,
    pub parent_node_id: Option<u32>,
    #[serde(default)]
    pub child_node_ids: Vec<u32>,
    pub cluster_start: u32,
    pub cluster_count: u32,
    pub page_id: u32,
    pub mip_level: u8,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub screen_space_error: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VirtualGeometryClusterHeaderAsset {
    pub cluster_id: u32,
    pub page_id: u32,
    pub hierarchy_node_id: u32,
    pub lod_level: u8,
    pub parent_cluster_id: Option<u32>,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub screen_space_error: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualGeometryClusterPageHeaderAsset {
    pub page_id: u32,
    pub start_offset: u32,
    pub payload_size_bytes: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualGeometryPageDependencyAsset {
    pub page_id: u32,
    pub parent_page_id: Option<u32>,
    #[serde(default)]
    pub child_page_ids: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualGeometryRootClusterRangeAsset {
    pub node_id: u32,
    pub cluster_start: u32,
    pub cluster_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualGeometryDebugMetadataAsset {
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct VirtualGeometryAsset {
    #[serde(default)]
    pub hierarchy_buffer: Vec<VirtualGeometryHierarchyNodeAsset>,
    #[serde(default)]
    pub cluster_headers: Vec<VirtualGeometryClusterHeaderAsset>,
    #[serde(default)]
    pub cluster_page_headers: Vec<VirtualGeometryClusterPageHeaderAsset>,
    #[serde(default)]
    pub cluster_page_data: Vec<Vec<u8>>,
    #[serde(default)]
    pub root_page_table: Vec<u32>,
    #[serde(default)]
    pub page_dependencies: Vec<VirtualGeometryPageDependencyAsset>,
    #[serde(default)]
    pub root_cluster_ranges: Vec<VirtualGeometryRootClusterRangeAsset>,
    #[serde(default)]
    pub debug: VirtualGeometryDebugMetadataAsset,
}

#[derive(Deserialize)]
struct VirtualGeometryAssetDocument {
    #[serde(default)]
    hierarchy_buffer: Vec<VirtualGeometryHierarchyNodeAsset>,
    #[serde(default)]
    cluster_headers: Vec<VirtualGeometryClusterHeaderAsset>,
    #[serde(default)]
    cluster_page_headers: Vec<VirtualGeometryClusterPageHeaderAsset>,
    #[serde(default)]
    cluster_page_data: Vec<Vec<u8>>,
    #[serde(default)]
    root_page_table: Vec<u32>,
    #[serde(default)]
    page_dependencies: Vec<VirtualGeometryPageDependencyAsset>,
    #[serde(default)]
    root_cluster_ranges: Vec<VirtualGeometryRootClusterRangeAsset>,
    #[serde(default)]
    debug: VirtualGeometryDebugMetadataAsset,
}

impl<'de> Deserialize<'de> for VirtualGeometryAsset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let document = VirtualGeometryAssetDocument::deserialize(deserializer)?;
        let asset = Self {
            hierarchy_buffer: document.hierarchy_buffer,
            cluster_headers: document.cluster_headers,
            cluster_page_headers: document.cluster_page_headers,
            cluster_page_data: document.cluster_page_data,
            root_page_table: document.root_page_table,
            page_dependencies: document.page_dependencies,
            root_cluster_ranges: document.root_cluster_ranges,
            debug: document.debug,
        };
        asset
            .validate_cluster_ranges()
            .map_err(serde::de::Error::custom)?;
        Ok(asset)
    }
}

impl VirtualGeometryAsset {
    fn validate_cluster_ranges(&self) -> Result<(), String> {
        let cluster_header_count = self.cluster_headers.len();
        for node in &self.hierarchy_buffer {
            validate_cluster_range(
                "hierarchy node",
                node.node_id,
                node.cluster_start,
                node.cluster_count,
                cluster_header_count,
            )?;
        }
        for range in &self.root_cluster_ranges {
            validate_cluster_range(
                "root range",
                range.node_id,
                range.cluster_start,
                range.cluster_count,
                cluster_header_count,
            )?;
        }
        Ok(())
    }
}

fn validate_cluster_range(
    kind: &str,
    id: u32,
    start: u32,
    count: u32,
    cluster_header_count: usize,
) -> Result<(), String> {
    let valid = usize::try_from(start)
        .ok()
        .zip(usize::try_from(count).ok())
        .and_then(|(start, count)| start.checked_add(count).map(|end| (start, end)))
        .is_some_and(|(start, end)| start <= cluster_header_count && end <= cluster_header_count);
    if valid {
        Ok(())
    } else {
        Err(format!(
            "{kind} {id} cluster range start={start} count={count} exceeds {cluster_header_count} headers"
        ))
    }
}
