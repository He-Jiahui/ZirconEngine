use serde::{Deserialize, Serialize};

use crate::asset::{AssetUri, MeshVertex};

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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
    pub root_cluster_ranges: Vec<VirtualGeometryRootClusterRangeAsset>,
    #[serde(default)]
    pub debug: VirtualGeometryDebugMetadataAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelPrimitiveAsset {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    #[serde(default)]
    pub virtual_geometry: Option<VirtualGeometryAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAsset {
    pub uri: AssetUri,
    pub primitives: Vec<ModelPrimitiveAsset>,
}

impl ModelAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
