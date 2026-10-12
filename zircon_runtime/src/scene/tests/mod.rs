//! 以真实世界行为和源码边界断言共同守护场景服务；锁恢复测试只保证服务仍可访问，不承诺中途写入会回滚。

#[path = "asset_scene.rs"]
mod asset_scene;
#[path = "authoring_boundary.rs"]
mod authoring_boundary;
#[path = "component_structure.rs"]
mod component_structure;
#[path = "derived_state.rs"]
mod derived_state;
#[cfg(feature = "physics-contracts")]
#[path = "dynamic_scene.rs"]
mod dynamic_scene;
#[path = "dynamic_scene_asset_reload.rs"]
mod dynamic_scene_asset_reload;
#[path = "session/mod.rs"]
mod dynamic_scene_session;
#[path = "ecs_archetype_index_structure.rs"]
mod ecs_archetype_index_structure;
#[path = "ecs_cached_query_iter_structure.rs"]
mod ecs_cached_query_iter_structure;
#[path = "ecs_change_detection.rs"]
mod ecs_change_detection;
#[path = "ecs_commands.rs"]
mod ecs_commands;
#[path = "ecs_component_storage_structure.rs"]
mod ecs_component_storage_structure;
#[path = "ecs_detached_entity_batch_performance.rs"]
mod ecs_detached_entity_batch_performance;
#[path = "ecs_dynamic_components_structure.rs"]
mod ecs_dynamic_components_structure;
#[path = "ecs_event_mirror.rs"]
mod ecs_event_mirror;
#[path = "ecs_events_messages.rs"]
mod ecs_events_messages;
#[path = "ecs_hierarchy_structure.rs"]
mod ecs_hierarchy_structure;
#[path = "ecs_identity_storage.rs"]
mod ecs_identity_storage;
#[path = "ecs_node_records_structure.rs"]
mod ecs_node_records_structure;
#[path = "ecs_observers_messages.rs"]
mod ecs_observers_messages;
#[path = "ecs_performance_acceptance.rs"]
mod ecs_performance_acceptance;
#[path = "ecs_query.rs"]
mod ecs_query;
#[path = "ecs_query_combinations.rs"]
mod ecs_query_combinations;
#[path = "ecs_query_data_structure.rs"]
mod ecs_query_data_structure;
#[path = "ecs_query_filter_structure.rs"]
mod ecs_query_filter_structure;
#[path = "ecs_query_many.rs"]
mod ecs_query_many;
#[path = "ecs_query_single.rs"]
mod ecs_query_single;
#[path = "ecs_query_state_structure.rs"]
mod ecs_query_state_structure;
#[path = "ecs_query_structure.rs"]
mod ecs_query_structure;
#[path = "ecs_reflect/mod.rs"]
mod ecs_reflect;
#[path = "ecs_schedule.rs"]
mod ecs_schedule;
#[path = "ecs_schedule_conflict_graph_structure.rs"]
mod ecs_schedule_conflict_graph_structure;
#[path = "ecs_schedule_parallel_executor_structure.rs"]
mod ecs_schedule_parallel_executor_structure;
#[path = "ecs_scheduled_native_systems.rs"]
mod ecs_scheduled_native_systems;
#[path = "ecs_system_query_cache.rs"]
mod ecs_system_query_cache;
#[path = "ecs_systems.rs"]
mod ecs_systems;
#[path = "ecs_typed_api.rs"]
mod ecs_typed_api;
#[path = "ecs_worker_command_buffers.rs"]
mod ecs_worker_command_buffers;
#[path = "inspection.rs"]
mod inspection;
#[path = "level_system_frame_state.rs"]
mod level_system_frame_state;
#[path = "physics_animation_components.rs"]
mod physics_animation_components;
#[path = "property_paths.rs"]
mod property_paths;
#[path = "render_dirty_journal/mod.rs"]
mod render_dirty_journal;
#[path = "render_extract.rs"]
mod render_extract;
#[path = "render_post_process_extract.rs"]
mod render_post_process_extract;
#[path = "semantics.rs"]
mod semantics;
#[path = "support.rs"]
mod support;
#[path = "world_basics.rs"]
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
    // BUG: [CR-SCENE-TEST-0001] 生命周期拆分后，此处仍匹配旧的局部句柄发布形式，并要求旧排序调用出现三次；当前发布使用已准备关卡的句柄，由有序注册表直接返回快照，故查找先失败。证据：level_manager_lifecycle 的准备与发布路径。
    let insert_index = normalized
        .find("self.lock_levels().insert(handle,level.clone());")
        .expect("level creation publishes the completed level");

    assert!(apply_index < insert_index);
    assert!(
        normalized.contains("letlevels=self.level_snapshots_in_handle_order();forlevelinlevels{")
    );
    assert_eq!(normalized.matches("sort_levels_by_handle(").count(), 3);
    assert!(normalized.contains("fetch_update(Ordering::Relaxed,Ordering::Relaxed,"));
    assert!(!normalized.contains("fetch_add(1,Ordering::Relaxed)+1"));
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
