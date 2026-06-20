mod asset_scene;
mod authoring_boundary;
mod component_structure;
mod derived_state;
mod dynamic_scene;
mod dynamic_scene_asset_reload;
mod dynamic_scene_session;
mod ecs_archetype_index_structure;
mod ecs_cached_query_iter_structure;
mod ecs_change_detection;
mod ecs_commands;
mod ecs_component_storage_structure;
mod ecs_dynamic_components_structure;
mod ecs_events_messages;
mod ecs_hierarchy_structure;
mod ecs_identity_storage;
mod ecs_node_records_structure;
mod ecs_observers_messages;
mod ecs_performance_acceptance;
mod ecs_query;
mod ecs_query_combinations;
mod ecs_query_data_structure;
mod ecs_query_filter_structure;
mod ecs_query_many;
mod ecs_query_single;
mod ecs_query_state_structure;
mod ecs_query_structure;
mod ecs_reflect;
mod ecs_schedule;
mod ecs_schedule_conflict_graph_structure;
mod ecs_schedule_parallel_executor_structure;
mod ecs_scheduled_native_systems;
mod ecs_system_query_cache;
mod ecs_systems;
mod ecs_typed_api;
mod inspection;
mod physics_animation_components;
mod property_paths;
mod render_extract;
mod render_post_process_extract;
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
