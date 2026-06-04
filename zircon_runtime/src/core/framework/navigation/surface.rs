use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::constants::{NavAreaId, AREA_WALKABLE, DEFAULT_AGENT_TYPE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshCollectMode {
    AllObjects,
    Hierarchy,
    Volume,
    ModifierOnly,
}

impl Default for NavMeshCollectMode {
    fn default() -> Self {
        Self::AllObjects
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshUseGeometry {
    RenderMeshes,
    PhysicsColliders,
}

impl Default for NavMeshUseGeometry {
    fn default() -> Self {
        Self::RenderMeshes
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshSurfaceDescriptor {
    pub enabled: bool,
    pub agent_type: String,
    pub collect_mode: NavMeshCollectMode,
    pub volume_center: [Real; 3],
    pub volume_size: [Real; 3],
    pub use_geometry: NavMeshUseGeometry,
    pub include_layers: Vec<String>,
    pub default_area: NavAreaId,
    pub generate_links: bool,
    pub override_voxel_size: Option<Real>,
    pub override_tile_size: Option<u32>,
    pub min_region_area: Real,
    pub build_height_mesh: bool,
    pub output_asset: Option<String>,
}

impl Default for NavMeshSurfaceDescriptor {
    fn default() -> Self {
        Self {
            enabled: true,
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            collect_mode: NavMeshCollectMode::AllObjects,
            volume_center: [0.0, 0.0, 0.0],
            volume_size: [10.0, 4.0, 10.0],
            use_geometry: NavMeshUseGeometry::RenderMeshes,
            include_layers: Vec::new(),
            default_area: AREA_WALKABLE,
            generate_links: true,
            override_voxel_size: None,
            override_tile_size: None,
            min_region_area: 2.0,
            build_height_mesh: false,
            output_asset: None,
        }
    }
}
