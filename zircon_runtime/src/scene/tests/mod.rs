mod asset_scene;
mod component_structure;
mod dynamic_scene;
mod ecs_change_detection;
mod ecs_identity_storage;
mod ecs_observers_messages;
mod ecs_performance_acceptance;
mod ecs_query;
mod ecs_query_combinations;
mod ecs_query_many;
mod ecs_query_single;
mod ecs_reflect;
mod ecs_schedule;
mod ecs_scheduled_native_systems;
mod ecs_system_query_cache;
mod ecs_systems;
mod ecs_typed_api;
mod editor_projection;
mod physics_animation_components;
mod property_paths;
mod semantics;
mod support;
mod world_basics;

use crate::scene::{DefaultLevelManager, RuntimeObject, RuntimeSystem};

#[test]
fn level_manager_produces_level_systems() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    assert!(manager.level(level.handle()).is_some());
}

#[test]
fn runtime_semantics_keep_ecs_roles_explicit() {
    let level = DefaultLevelManager::default().create_default_level();

    assert_eq!(level.object_kind(), "system");
    assert_eq!(level.system_name(), "LevelSystem");
}
