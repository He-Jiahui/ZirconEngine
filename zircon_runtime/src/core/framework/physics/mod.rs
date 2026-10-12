//! Physics framework contracts for backend status, world stepping, queries, contacts, and scene sync.
//! 物理框架只定义场景与插件共享的状态、查询和逐帧载荷；后端选择及求解由物理插件负责。

#[path = "backend_state.rs"]
mod backend_state;
#[path = "backend_status.rs"]
mod backend_status;
#[path = "body_sync_state.rs"]
mod body_sync_state;
#[path = "body_type.rs"]
mod body_type;
#[path = "collider_shape.rs"]
mod collider_shape;
#[path = "collider_sync_state.rs"]
mod collider_sync_state;
#[path = "contact_event.rs"]
mod contact_event;
#[path = "joint_sync_state.rs"]
mod joint_sync_state;
#[path = "joint_type.rs"]
mod joint_type;
#[path = "manager.rs"]
mod manager;
#[path = "material_sync_state.rs"]
mod material_sync_state;
#[path = "mesh_asset.rs"]
mod mesh_asset;
#[path = "query_filter.rs"]
mod query_filter;
#[path = "query_interface.rs"]
mod query_interface;
#[path = "query_mode.rs"]
mod query_mode;
#[path = "ray_cast_hit.rs"]
mod ray_cast_hit;
#[path = "ray_cast_query.rs"]
mod ray_cast_query;
#[path = "scene_step_result.rs"]
mod scene_step_result;
#[path = "settings.rs"]
mod settings;
#[path = "settings_store_error.rs"]
mod settings_store_error;
#[path = "shape_cast_hit.rs"]
mod shape_cast_hit;
#[path = "shape_cast_query.rs"]
mod shape_cast_query;
#[path = "shape_overlap_hit.rs"]
mod shape_overlap_hit;
#[path = "shape_overlap_query.rs"]
mod shape_overlap_query;
#[path = "simulation_mode.rs"]
mod simulation_mode;
#[path = "skeletal_pose.rs"]
mod skeletal_pose;
#[path = "trigger_event.rs"]
mod trigger_event;
#[path = "trigger_event_kind.rs"]
mod trigger_event_kind;
#[path = "world_step_plan.rs"]
mod world_step_plan;
#[path = "world_sync_state.rs"]
mod world_sync_state;

pub use backend_state::PhysicsBackendState;
pub use backend_status::PhysicsBackendStatus;
pub use body_sync_state::PhysicsBodySyncState;
pub use body_type::PhysicsBodyType;
pub use collider_shape::PhysicsColliderShape;
pub use collider_sync_state::PhysicsColliderSyncState;
pub use contact_event::PhysicsContactEvent;
pub use joint_sync_state::PhysicsJointSyncState;
pub use joint_type::PhysicsJointType;
pub use manager::PhysicsManager;
pub use material_sync_state::PhysicsMaterialSyncState;
pub use mesh_asset::PhysicsMeshAsset;
pub use query_filter::PhysicsQueryFilter;
pub use query_interface::{PhysicsQueryInterface, PHYSICS_QUERY_INTERFACE_ID};
pub use query_mode::PhysicsQueryMode;
pub use ray_cast_hit::PhysicsRayCastHit;
pub use ray_cast_query::PhysicsRayCastQuery;
pub use scene_step_result::PhysicsSceneStepResult;
pub use settings::PhysicsSettings;
pub use settings_store_error::PhysicsSettingsStoreError;
pub use shape_cast_hit::PhysicsShapeCastHit;
pub use shape_cast_query::PhysicsShapeCastQuery;
pub use shape_overlap_hit::PhysicsShapeOverlapHit;
pub use shape_overlap_query::PhysicsShapeOverlapQuery;
pub use simulation_mode::PhysicsSimulationMode;
pub use skeletal_pose::{SimulatedPoseFeed, SkeletalPoseTarget, SkeletalPoseTargets};
pub use trigger_event::PhysicsTriggerEvent;
pub use trigger_event_kind::PhysicsTriggerEventKind;
pub use world_step_plan::PhysicsWorldStepPlan;
pub use world_sync_state::PhysicsWorldSyncState;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
