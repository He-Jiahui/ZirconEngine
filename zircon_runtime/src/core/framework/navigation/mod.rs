//! Neutral navigation contracts shared by runtime plugins and editor tools.

mod agent;
mod asset;
mod bake;
mod constants;
mod error;
mod gizmo;
mod handle;
mod manager;
mod modifier;
mod obstacle;
mod off_mesh_link;
mod operation;
mod query;
mod settings;
mod stats;
mod surface;

pub use agent::{
    NavAgentTickReport, NavAgentWritebackMode, NavAvoidanceQuality, NavDesiredVelocity,
    NavMeshAgentDescriptor, NavigationAgentDebugState, NavigationDebugCapture,
};
pub use asset::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshGizmoTriangleAsset, NavMeshLinkAsset,
    NavMeshLinkCapacity, NavMeshPolygonAsset, NavMeshTileAsset, NavigationAssetError,
    NavigationAssetResult, NavigationSettingsAsset,
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
    NavLinkMotion, NavLinkTraversalMode, NavMeshOffMeshBridgeDescriptor,
    NavMeshOffMeshLinkDescriptor, OffMeshTraverseEvent, OffMeshTraverseEventKind,
    OffMeshTraversePhase, OffMeshTraverseState,
};
pub use operation::{
    NavigationClearBakeRequest, NavigationGeneratedBakeChange, NavigationGeneratedBakeSnapshot,
    NAVIGATION_BAKE_SCENE_OPERATION, NAVIGATION_BAKE_SURFACE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION, NAVIGATION_RESTORE_BAKE_OPERATION,
};
pub use query::{
    nav_area_flag, NavPathPoint, NavPathQuery, NavPathResult, NavPathStatus, NavQueryFilter,
    NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery,
};
pub use settings::{default_navigation_areas, NavigationAgentSettings, NavigationAreaSettings};
pub use stats::NavigationRuntimeStats;
pub use surface::{NavMeshCollectMode, NavMeshSurfaceDescriptor, NavMeshUseGeometry};

#[cfg(test)]
mod tests;
