mod asset_scene;
mod authoring_boundary;
mod component_structure;
mod derived_state;
#[cfg(feature = "physics-contracts")]
mod dynamic_scene;
mod dynamic_scene_asset_reload;
mod dynamic_scene_session;
mod ecs_archetype_index_structure;
mod ecs_cached_query_iter_structure;
mod ecs_change_detection;
mod ecs_commands;
mod ecs_component_storage_structure;
mod ecs_detached_entity_batch_performance;
mod ecs_dynamic_components_structure;
mod ecs_event_mirror;
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
mod ecs_worker_command_buffers;
mod inspection;
mod level_system_frame_state;
mod physics_animation_components;
mod property_paths;
mod render_extract;
mod render_post_process_extract;
mod semantics;
mod support;
mod world_basics;

use crate::scene::{DefaultLevelManager, RuntimeObject, RuntimeSystem};

#[test]
fn level_system_state_locks_use_poison_recovery_helpers() {
    fn production_source(source: &str) -> &str {
        source.split("\n#[cfg(test)]").next().unwrap_or(source)
    }

    let sources = [
        production_source(include_str!("../level_system.rs")),
        include_str!("../module/default_level_manager.rs"),
        include_str!("../module/level_manager_lifecycle.rs"),
    ]
    .join("\n");
    let normalized_sources: String = sources
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();

    assert!(normalized_sources.contains(".lock().unwrap_or_else("));
    assert!(!normalized_sources.contains(".lock().unwrap("));
}

#[test]
fn level_manager_produces_level_systems() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    assert!(manager.level(level.handle()).is_some());
}

#[test]
fn level_manager_registry_locks_do_not_cross_world_work() {
    let contract_source = include_str!("../module/level_manager_contract.rs");
    assert!(contract_source.contains("self.lock_levels().contains_key(&handle)"));

    let lifecycle_source = include_str!("../module/level_manager_lifecycle.rs");
    let normalized: String = lifecycle_source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let apply_index = normalized
        .find("driver.apply_world_runtime_extensions(&mutworld)?;")
        .expect("level creation applies runtime extensions");
    let insert_index = normalized
        .find("self.lock_levels().insert(handle,level.clone());")
        .expect("level creation publishes the completed level");

    assert!(apply_index < insert_index);
    assert!(normalized.contains(
        "letlevels=self.lock_levels().values().cloned().collect::<Vec<_>>();forlevelinlevels{"
    ));
    assert!(normalized.contains("fetch_add(1,Ordering::Relaxed)"));
}

#[test]
fn level_system_recovers_world_lock_after_writer_panic() {
    let level = DefaultLevelManager::default().create_default_level();
    let poison_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        level.with_world_mut(|_| panic!("poison level world lock"));
    }));

    assert!(poison_result.is_err());
    let _snapshot_after_poison = level.snapshot();
}

#[test]
fn runtime_semantics_keep_ecs_roles_explicit() {
    let level = DefaultLevelManager::default().create_default_level();

    assert_eq!(level.object_kind(), "system");
    assert_eq!(level.system_name(), "LevelSystem");
}
