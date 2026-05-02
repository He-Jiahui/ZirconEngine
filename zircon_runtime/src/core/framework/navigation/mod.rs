//! Neutral navigation contracts shared by runtime plugins and editor tools.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::asset::{NavMeshAsset, NavigationSettingsAsset};
use crate::core::framework::render::{
    OverlayLineSegment, OverlayPickShape, SceneGizmoKind, SceneGizmoOverlayExtract,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::Real;
use crate::core::math::{Vec3, Vec4};
use crate::scene::World;

pub const NAV_MESH_SURFACE_COMPONENT_TYPE: &str = "navigation.Component.NavMeshSurface";
pub const NAV_MESH_MODIFIER_COMPONENT_TYPE: &str = "navigation.Component.NavMeshModifier";
pub const NAV_MESH_AGENT_COMPONENT_TYPE: &str = "navigation.Component.NavMeshAgent";
pub const NAV_MESH_OBSTACLE_COMPONENT_TYPE: &str = "navigation.Component.NavMeshObstacle";
pub const NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE: &str = "navigation.Component.NavMeshOffMeshLink";

pub const DEFAULT_AGENT_TYPE: &str = "humanoid";
pub const AREA_NOT_WALKABLE: NavAreaId = 0;
pub const AREA_WALKABLE: NavAreaId = 1;
pub const AREA_JUMP: NavAreaId = 2;
pub const CUSTOM_AREA_START: NavAreaId = 3;
pub const MAX_NAV_AREAS: usize = 64;
pub const DEFAULT_AREA_MASK: NavAreaMask = u64::MAX;

pub type NavAreaId = u8;
pub type NavAreaMask = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NavMeshHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationAgentSettings {
    pub id: String,
    pub display_name: String,
    pub radius: Real,
    pub height: Real,
    pub max_climb: Real,
    pub max_slope_degrees: Real,
    pub speed: Real,
    pub acceleration: Real,
    pub angular_speed_degrees: Real,
    pub stopping_distance: Real,
}

impl NavigationAgentSettings {
    pub fn humanoid() -> Self {
        Self {
            id: DEFAULT_AGENT_TYPE.to_string(),
            display_name: "Humanoid".to_string(),
            radius: 0.5,
            height: 2.0,
            max_climb: 0.4,
            max_slope_degrees: 45.0,
            speed: 3.5,
            acceleration: 8.0,
            angular_speed_degrees: 360.0,
            stopping_distance: 0.1,
        }
    }
}

impl Default for NavigationAgentSettings {
    fn default() -> Self {
        Self::humanoid()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationAreaSettings {
    pub id: NavAreaId,
    pub name: String,
    pub cost: Real,
    pub walkable: bool,
}

impl NavigationAreaSettings {
    pub fn not_walkable() -> Self {
        Self {
            id: AREA_NOT_WALKABLE,
            name: "not_walkable".to_string(),
            cost: 0.0,
            walkable: false,
        }
    }

    pub fn walkable() -> Self {
        Self {
            id: AREA_WALKABLE,
            name: "walkable".to_string(),
            cost: 1.0,
            walkable: true,
        }
    }

    pub fn jump() -> Self {
        Self {
            id: AREA_JUMP,
            name: "jump".to_string(),
            cost: 2.0,
            walkable: true,
        }
    }
}

pub fn default_navigation_areas() -> Vec<NavigationAreaSettings> {
    vec![
        NavigationAreaSettings::not_walkable(),
        NavigationAreaSettings::walkable(),
        NavigationAreaSettings::jump(),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshModifierMode {
    Add,
    Modify,
    Remove,
}

impl Default for NavMeshModifierMode {
    fn default() -> Self {
        Self::Modify
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshModifierDescriptor {
    pub mode: NavMeshModifierMode,
    pub affected_agents: Vec<String>,
    pub apply_to_children: bool,
    pub override_area: bool,
    pub area: NavAreaId,
    pub override_generate_links: bool,
    pub generate_links: bool,
}

impl Default for NavMeshModifierDescriptor {
    fn default() -> Self {
        Self {
            mode: NavMeshModifierMode::Modify,
            affected_agents: Vec::new(),
            apply_to_children: true,
            override_area: false,
            area: AREA_WALKABLE,
            override_generate_links: false,
            generate_links: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavAvoidanceQuality {
    None,
    Low,
    Medium,
    High,
}

impl Default for NavAvoidanceQuality {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshAgentDescriptor {
    pub agent_type: String,
    pub radius: Real,
    pub height: Real,
    pub base_offset: Real,
    pub speed: Real,
    pub angular_speed: Real,
    pub acceleration: Real,
    pub stopping_distance: Real,
    pub auto_braking: bool,
    pub avoidance_quality: NavAvoidanceQuality,
    pub priority: u8,
    pub area_mask: NavAreaMask,
    pub auto_repath: bool,
    pub auto_traverse_links: bool,
    pub update_position: bool,
    pub update_rotation: bool,
    pub destination: Option<[Real; 3]>,
}

impl Default for NavMeshAgentDescriptor {
    fn default() -> Self {
        let agent = NavigationAgentSettings::humanoid();
        Self {
            agent_type: agent.id,
            radius: agent.radius,
            height: agent.height,
            base_offset: 0.0,
            speed: agent.speed,
            angular_speed: agent.angular_speed_degrees,
            acceleration: agent.acceleration,
            stopping_distance: agent.stopping_distance,
            auto_braking: true,
            avoidance_quality: NavAvoidanceQuality::Medium,
            priority: 50,
            area_mask: DEFAULT_AREA_MASK,
            auto_repath: true,
            auto_traverse_links: true,
            update_position: true,
            update_rotation: true,
            destination: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshObstacleShape {
    Box,
    Capsule,
}

impl Default for NavMeshObstacleShape {
    fn default() -> Self {
        Self::Box
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshObstacleDescriptor {
    pub shape: NavMeshObstacleShape,
    pub center: [Real; 3],
    pub size: [Real; 3],
    pub radius: Real,
    pub height: Real,
    pub avoidance_enabled: bool,
    pub carve: bool,
    pub move_threshold: Real,
    pub time_to_stationary: Real,
    pub carve_only_stationary: bool,
}

impl Default for NavMeshObstacleDescriptor {
    fn default() -> Self {
        Self {
            shape: NavMeshObstacleShape::Box,
            center: [0.0, 0.0, 0.0],
            size: [1.0, 1.0, 1.0],
            radius: 0.5,
            height: 2.0,
            avoidance_enabled: true,
            carve: false,
            move_threshold: 0.1,
            time_to_stationary: 0.5,
            carve_only_stationary: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavLinkTraversalMode {
    Automatic,
    Manual,
}

impl Default for NavLinkTraversalMode {
    fn default() -> Self {
        Self::Automatic
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshOffMeshLinkDescriptor {
    pub start_entity: Option<u64>,
    pub end_entity: Option<u64>,
    pub start_local_point: [Real; 3],
    pub end_local_point: [Real; 3],
    pub width: Real,
    pub bidirectional: bool,
    pub activated: bool,
    pub auto_update_positions: bool,
    pub cost_override: Option<Real>,
    pub area_type: NavAreaId,
    pub agent_type: String,
    pub traversal_mode: NavLinkTraversalMode,
}

impl Default for NavMeshOffMeshLinkDescriptor {
    fn default() -> Self {
        Self {
            start_entity: None,
            end_entity: None,
            start_local_point: [0.0, 0.0, 0.0],
            end_local_point: [0.0, 0.0, 1.0],
            width: 0.0,
            bidirectional: true,
            activated: true,
            auto_update_positions: true,
            cost_override: None,
            area_type: AREA_JUMP,
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            traversal_mode: NavLinkTraversalMode::Automatic,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshBakeRequest {
    pub surface_entity: Option<u64>,
    pub agent_type: Option<String>,
    pub output_asset: Option<String>,
    pub force_full_rebuild: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshBakeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshBakeDiagnostic {
    pub severity: NavMeshBakeDiagnosticSeverity,
    pub message: String,
    pub entity: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavMeshBakeReport {
    pub asset: Option<NavMeshAsset>,
    pub output_asset: Option<String>,
    pub surfaces: usize,
    pub source_vertices: usize,
    pub source_triangles: usize,
    pub baked_vertices: usize,
    pub baked_polygons: usize,
    pub tiles: usize,
    pub diagnostics: Vec<NavMeshBakeDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavPathQuery {
    pub nav_mesh: Option<NavMeshHandle>,
    pub start: [Real; 3],
    pub end: [Real; 3],
    pub agent_type: String,
    pub area_mask: NavAreaMask,
}

impl NavPathQuery {
    pub fn new(start: [Real; 3], end: [Real; 3]) -> Self {
        Self {
            nav_mesh: None,
            start,
            end,
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            area_mask: DEFAULT_AREA_MASK,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavPathStatus {
    Complete,
    Partial,
    NoPath,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavPathPoint {
    pub position: [Real; 3],
    pub area: NavAreaId,
    pub flags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavPathResult {
    pub status: NavPathStatus,
    pub points: Vec<NavPathPoint>,
    pub length: Real,
    pub visited_nodes: usize,
}

impl NavPathResult {
    pub fn no_path() -> Self {
        Self {
            status: NavPathStatus::NoPath,
            points: Vec::new(),
            length: 0.0,
            visited_nodes: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavSampleQuery {
    pub nav_mesh: Option<NavMeshHandle>,
    pub position: [Real; 3],
    pub extents: [Real; 3],
    pub agent_type: String,
    pub area_mask: NavAreaMask,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavSampleHit {
    pub position: [Real; 3],
    pub distance: Real,
    pub area: NavAreaId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavRaycastQuery {
    pub nav_mesh: Option<NavMeshHandle>,
    pub start: [Real; 3],
    pub end: [Real; 3],
    pub agent_type: String,
    pub area_mask: NavAreaMask,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavRaycastResult {
    pub hit: bool,
    pub position: [Real; 3],
    pub normal: [Real; 3],
    pub distance: Real,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavAgentTickReport {
    pub scanned_agents: usize,
    pub moved_agents: usize,
    pub blocked_agents: usize,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationRuntimeStats {
    pub loaded_nav_meshes: usize,
    pub active_agents: usize,
    pub active_obstacles: usize,
    pub active_off_mesh_links: usize,
}

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

    pub fn to_scene_gizmo_overlay(
        &self,
        owner: EntityId,
        selected: bool,
    ) -> SceneGizmoOverlayExtract {
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
        SceneGizmoOverlayExtract {
            owner,
            kind: SceneGizmoKind::NavigationMesh,
            selected,
            lines,
            wire_shapes: Vec::new(),
            icons: Vec::new(),
            pick_shapes,
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationErrorKind {
    InvalidConfiguration,
    MissingNavMesh,
    NoPath,
    BakeFailed,
    BackendFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationError {
    pub kind: NavigationErrorKind,
    pub message: String,
}

impl NavigationError {
    pub fn new(kind: NavigationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn missing_nav_mesh(message: impl Into<String>) -> Self {
        Self::new(NavigationErrorKind::MissingNavMesh, message)
    }
}

impl fmt::Display for NavigationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.message)
    }
}

impl Error for NavigationError {}

pub trait NavigationManager: Send + Sync {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError>;

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError>;

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError>;

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError>;

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError>;

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError>;

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError>;

    fn stats(&self) -> NavigationRuntimeStats;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_humanoid_agent_contract() {
        let agent = NavigationAgentSettings::humanoid();
        assert_eq!(agent.id, "humanoid");
        assert_eq!(agent.radius, 0.5);
        assert_eq!(agent.height, 2.0);
        assert_eq!(agent.max_climb, 0.4);
        assert_eq!(agent.max_slope_degrees, 45.0);
        assert_eq!(agent.speed, 3.5);
        assert_eq!(agent.acceleration, 8.0);
        assert_eq!(agent.angular_speed_degrees, 360.0);
        assert_eq!(agent.stopping_distance, 0.1);
    }

    #[test]
    fn component_type_ids_are_plugin_prefixed() {
        for type_id in [
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            NAV_MESH_MODIFIER_COMPONENT_TYPE,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        ] {
            assert!(type_id.starts_with("navigation.Component."));
        }
    }

    #[test]
    fn nav_mesh_asset_gizmo_snapshot_projects_triangle_edges() {
        let snapshot = NavigationGizmoSnapshot::from_nav_mesh_asset(&NavMeshAsset::simple_quad(
            DEFAULT_AGENT_TYPE,
            2.0,
        ));
        let overlay = snapshot.to_scene_gizmo_overlay(42, true);

        assert_eq!(snapshot.triangles.len(), 2);
        assert_eq!(overlay.owner, 42);
        assert_eq!(overlay.lines.len(), 6);
        assert!(overlay.selected);
    }
}
