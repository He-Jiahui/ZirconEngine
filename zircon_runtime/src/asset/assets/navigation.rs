use serde::{Deserialize, Serialize};

use crate::core::framework::navigation::{
    default_navigation_areas, NavAreaId, NavLinkTraversalMode, NavigationAgentSettings,
    NavigationAreaSettings, AREA_WALKABLE, DEFAULT_AGENT_TYPE,
};
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavMeshAsset {
    pub version: u32,
    pub agent_type: String,
    pub settings_hash: u64,
    pub vertices: Vec<[Real; 3]>,
    pub indices: Vec<u32>,
    pub polygons: Vec<NavMeshPolygonAsset>,
    pub tiles: Vec<NavMeshTileAsset>,
    pub off_mesh_links: Vec<NavMeshLinkAsset>,
}

impl NavMeshAsset {
    pub const VERSION: u32 = 1;

    pub fn empty(agent_type: impl Into<String>) -> Self {
        Self {
            version: Self::VERSION,
            agent_type: agent_type.into(),
            settings_hash: 0,
            vertices: Vec::new(),
            indices: Vec::new(),
            polygons: Vec::new(),
            tiles: Vec::new(),
            off_mesh_links: Vec::new(),
        }
    }

    pub fn simple_quad(agent_type: impl Into<String>, half_extent: Real) -> Self {
        let half_extent = half_extent.max(0.1);
        Self {
            version: Self::VERSION,
            agent_type: agent_type.into(),
            settings_hash: 0,
            vertices: vec![
                [-half_extent, 0.0, -half_extent],
                [half_extent, 0.0, -half_extent],
                [half_extent, 0.0, half_extent],
                [-half_extent, 0.0, half_extent],
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            polygons: vec![NavMeshPolygonAsset {
                first_index: 0,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 0,
            }],
            tiles: vec![NavMeshTileAsset {
                id: 0,
                bounds_min: [-half_extent, 0.0, -half_extent],
                bounds_max: [half_extent, 0.0, half_extent],
                polygon_count: 1,
            }],
            off_mesh_links: Vec::new(),
        }
    }

    pub fn from_triangle_mesh(
        agent_type: impl Into<String>,
        vertices: Vec<[Real; 3]>,
        indices: Vec<u32>,
        area: NavAreaId,
    ) -> Self {
        Self::from_triangle_mesh_with_areas(agent_type, vertices, indices, Vec::new(), area)
    }

    pub fn from_triangle_mesh_with_areas(
        agent_type: impl Into<String>,
        vertices: Vec<[Real; 3]>,
        indices: Vec<u32>,
        triangle_areas: Vec<NavAreaId>,
        default_area: NavAreaId,
    ) -> Self {
        let mut valid_indices = Vec::new();
        let mut polygons = Vec::new();
        for (triangle_index, triangle) in indices.chunks(3).enumerate() {
            if triangle.len() != 3
                || !triangle
                    .iter()
                    .all(|index| (*index as usize) < vertices.len())
            {
                continue;
            }
            let first_index = valid_indices.len() as u32;
            valid_indices.extend_from_slice(triangle);
            polygons.push(NavMeshPolygonAsset {
                first_index,
                index_count: 3,
                area: triangle_areas
                    .get(triangle_index)
                    .copied()
                    .unwrap_or(default_area),
                tile: 0,
            });
        }
        if vertices.is_empty() || valid_indices.is_empty() {
            return Self::empty(agent_type);
        }

        let (bounds_min, bounds_max) =
            nav_mesh_asset_bounds(&vertices).unwrap_or(([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]));

        Self {
            version: Self::VERSION,
            agent_type: agent_type.into(),
            settings_hash: 0,
            vertices,
            indices: valid_indices,
            tiles: vec![NavMeshTileAsset {
                id: 0,
                bounds_min,
                bounds_max,
                polygon_count: polygons.len() as u32,
            }],
            polygons,
            off_mesh_links: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.polygons.is_empty()
    }

    pub fn debug_triangles(&self) -> Vec<NavMeshGizmoTriangleAsset> {
        self.polygons
            .iter()
            .flat_map(|polygon| {
                let start = polygon.first_index as usize;
                let end = start.saturating_add(polygon.index_count as usize);
                self.indices[start.min(self.indices.len())..end.min(self.indices.len())]
                    .chunks(3)
                    .filter_map(move |indices| {
                        if indices.len() != 3 {
                            return None;
                        }
                        Some(NavMeshGizmoTriangleAsset {
                            vertices: [
                                *self.vertices.get(indices[0] as usize)?,
                                *self.vertices.get(indices[1] as usize)?,
                                *self.vertices.get(indices[2] as usize)?,
                            ],
                            area: polygon.area,
                            tile: polygon.tile,
                        })
                    })
            })
            .collect()
    }
}

fn nav_mesh_asset_bounds(vertices: &[[Real; 3]]) -> Option<([Real; 3], [Real; 3])> {
    let mut vertices = vertices.iter();
    let first = *vertices.next()?;
    let mut min = first;
    let mut max = first;
    for vertex in vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(vertex[axis]);
            max[axis] = max[axis].max(vertex[axis]);
        }
    }
    Some((min, max))
}

impl Default for NavMeshAsset {
    fn default() -> Self {
        Self::empty(DEFAULT_AGENT_TYPE)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavMeshPolygonAsset {
    pub first_index: u32,
    pub index_count: u32,
    pub area: NavAreaId,
    pub tile: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavMeshTileAsset {
    pub id: u32,
    pub bounds_min: [Real; 3],
    pub bounds_max: [Real; 3],
    pub polygon_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavMeshLinkAsset {
    pub start: [Real; 3],
    pub end: [Real; 3],
    pub width: Real,
    pub bidirectional: bool,
    pub area: NavAreaId,
    pub cost_override: Option<Real>,
    pub traversal_mode: NavLinkTraversalMode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavMeshGizmoTriangleAsset {
    pub vertices: [[Real; 3]; 3],
    pub area: NavAreaId,
    pub tile: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationSettingsAsset {
    pub version: u32,
    pub agents: Vec<NavigationAgentSettings>,
    pub areas: Vec<NavigationAreaSettings>,
}

impl NavigationSettingsAsset {
    pub const VERSION: u32 = 1;

    pub fn default_3d() -> Self {
        Self {
            version: Self::VERSION,
            agents: vec![NavigationAgentSettings::humanoid()],
            areas: default_navigation_areas(),
        }
    }

    pub fn agent(&self, id: &str) -> Option<&NavigationAgentSettings> {
        self.agents.iter().find(|agent| agent.id == id)
    }

    pub fn area(&self, id: NavAreaId) -> Option<&NavigationAreaSettings> {
        self.areas.iter().find(|area| area.id == id)
    }
}

impl Default for NavigationSettingsAsset {
    fn default() -> Self {
        Self::default_3d()
    }
}
