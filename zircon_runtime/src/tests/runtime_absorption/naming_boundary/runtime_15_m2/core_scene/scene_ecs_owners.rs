use super::*;

#[path = "scene_ecs_owners/component_storage_component_results.rs"]
mod component_storage_component_results;
#[path = "scene_ecs_owners/observer_callback_registry.rs"]
mod observer_callback_registry;
#[path = "scene_ecs_owners/query_state_many_item_array.rs"]
mod query_state_many_item_array;
#[path = "scene_ecs_owners/split_layout.rs"]
mod split_layout;

const CHILD_OWNER_STATUS: &str =
    "runtime_15_core_scene_naming_ecs_owner_guard_child_owner_split_static_passed_cargo_deferred";
const CHILD_OWNER_SLICE: &str = "Runtime 15 M3 core-scene naming ECS owner guard child-owner split";
const CHILD_OWNER_GUARD: &str = "runtime_15_core_scene_naming_ecs_owner_guards_are_child_owner";

const SPLIT_LAYOUT_STATUS: &str =
    "runtime_15_core_scene_naming_ecs_owner_split_layout_folder_backed_static_passed_cargo_deferred";
const SPLIT_LAYOUT_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_core_scene_naming_ecs_owner_split_layout_folder_backed_static_passed_cargo_deferred";
const SPLIT_LAYOUT_SLICE: &str =
    "Runtime 15 M3 core-scene naming ECS owner split-layout folder-backed split";
const SPLIT_LAYOUT_GUARD: &str =
    "runtime_15_core_scene_naming_ecs_owner_split_layout_is_folder_backed";

const PARENT_PATH: &str = "naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs";
const CHILD_PATHS: &[&str] = &[
    "naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/observer_callback_registry.rs",
    "naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/query_state_many_item_array.rs",
    "naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/component_storage_component_results.rs",
    "naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/split_layout.rs",
];
