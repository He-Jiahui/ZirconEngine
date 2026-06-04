use serde::{Deserialize, Serialize};

use crate::asset::NavMeshAsset;
use crate::core::framework::render::{
    OverlayLineSegment, OverlayPickShape, SceneGizmoKind,
    SceneGizmoOverlayExtract as SceneGizmoOverlay,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3, Vec4};

use super::constants::{NavAreaId, AREA_JUMP, AREA_NOT_WALKABLE, AREA_WALKABLE};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationGizmoSnapshot {
    pub triangles: Vec<NavigationGizmoTriangle>,
    pub off_mesh_links: Vec<NavigationGizmoLink>,
}

impl NavigationGizmoSnapshot {
    pub fn from_nav_mesh_asset(asset: &NavMeshAsset) -> Self {
        Self {
            triangles: asset
                .debug_triangles()
                .into_iter()
                .map(|triangle| NavigationGizmoTriangle {
                    vertices: triangle.vertices,
                    area: triangle.area,
                    tile: triangle.tile,
                })
                .collect(),
            off_mesh_links: asset
                .off_mesh_links
                .iter()
                .map(|link| NavigationGizmoLink {
                    start: link.start,
                    end: link.end,
                    area: link.area,
                    bidirectional: link.bidirectional,
                })
                .collect(),
        }
    }

    pub fn to_scene_gizmo_overlay(&self, owner: EntityId, selected: bool) -> SceneGizmoOverlay {
        let mut lines = Vec::new();
        for triangle in &self.triangles {
            let color = navigation_area_color(triangle.area);
            let vertices = triangle.vertices.map(Vec3::from_array);
            lines.push(OverlayLineSegment {
                start: vertices[0],
                end: vertices[1],
                color,
            });
            lines.push(OverlayLineSegment {
                start: vertices[1],
                end: vertices[2],
                color,
            });
            lines.push(OverlayLineSegment {
                start: vertices[2],
                end: vertices[0],
                color,
            });
        }
        let mut pick_shapes = Vec::new();
        for link in &self.off_mesh_links {
            let color = navigation_area_color(link.area);
            let start = Vec3::from_array(link.start);
            let end = Vec3::from_array(link.end);
            lines.push(OverlayLineSegment { start, end, color });
            pick_shapes.push(OverlayPickShape::Segment {
                start,
                end,
                thickness: 0.08,
            });
        }
        SceneGizmoOverlay::new(
            owner,
            SceneGizmoKind::NavigationMesh,
            selected,
            lines,
            Vec::new(),
            Vec::new(),
            pick_shapes,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationGizmoTriangle {
    pub vertices: [[Real; 3]; 3],
    pub area: NavAreaId,
    pub tile: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationGizmoLink {
    pub start: [Real; 3],
    pub end: [Real; 3],
    pub area: NavAreaId,
    pub bidirectional: bool,
}

fn navigation_area_color(area: NavAreaId) -> Vec4 {
    match area {
        AREA_NOT_WALKABLE => Vec4::new(0.85, 0.12, 0.12, 0.9),
        AREA_WALKABLE => Vec4::new(0.15, 0.78, 0.42, 0.9),
        AREA_JUMP => Vec4::new(0.25, 0.55, 1.0, 0.9),
        _ => Vec4::new(0.96, 0.72, 0.2, 0.9),
    }
}
