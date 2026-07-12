//! Neutral navigation contracts shared by runtime plugins and editor tools.

mod agent;
mod bake;
mod constants;
mod error;
mod gizmo;
mod handle;
mod manager;
mod modifier;
mod obstacle;
mod off_mesh_link;
mod query;
mod settings;
mod stats;
mod surface;

pub use agent::{
    NavAgentTickReport, NavAgentWritebackMode, NavAvoidanceQuality, NavDesiredVelocity,
    NavMeshAgentDescriptor,
};
pub use bake::{
    NavMeshBakeDiagnostic, NavMeshBakeDiagnosticSeverity, NavMeshBakeReport, NavMeshBakeRequest,
};
pub use constants::{
    NavAreaId, NavAreaMask, AREA_JUMP, AREA_NOT_WALKABLE, AREA_WALKABLE, CUSTOM_AREA_START,
    DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK, MAX_NAV_AREAS, MAX_OFF_MESH_BRIDGE_LANES,
    NAV_DESIRED_VELOCITY_COMPONENT_TYPE, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_MODIFIER_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
pub use error::{NavigationError, NavigationErrorKind};
pub use gizmo::{NavigationGizmoLink, NavigationGizmoSnapshot, NavigationGizmoTriangle};
pub use handle::NavMeshHandle;
pub use manager::NavigationManager;
pub use modifier::{NavMeshModifierDescriptor, NavMeshModifierMode};
pub use obstacle::{NavMeshObstacleDescriptor, NavMeshObstacleShape};
pub use off_mesh_link::{
    NavLinkTraversalMode, NavMeshOffMeshBridgeDescriptor, NavMeshOffMeshLinkDescriptor,
};
pub use query::{
    NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus, NavRaycastQuery, NavRaycastResult,
    NavSampleHit, NavSampleQuery,
};
pub use settings::{default_navigation_areas, NavigationAgentSettings, NavigationAreaSettings};
pub use stats::NavigationRuntimeStats;
pub use surface::{NavMeshCollectMode, NavMeshSurfaceDescriptor, NavMeshUseGeometry};

#[cfg(test)]
mod tests;
