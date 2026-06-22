use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 08 ECS 数据面镜像文档守卫",
        [
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
            "ecs_kernel_data_boundary",
            "standalone rustc 1/1",
            "entity/observer/command/messages/change_tick/ecs Cargo gates pending",
        ],
    ),
    (
        "Runtime 08 First-stage event update guard",
        [
            "first_stage_updates_all_registered_event_channels",
            "event_message_anchors = 12/12",
            "runtime_08_guard_anchors = 21/21",
            "standalone ecs_kernel_data 1/1",
        ],
    ),
    (
        "Runtime 08 ECS 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 16",
            "missing_behavior_test_anchors = []",
            "runtime_08_guard_anchors = 21/21",
            "standalone ecs_kernel_data 1/1",
        ],
    ),
    (
        "Runtime 08 ECS 数据面 current audit recheck",
        [
            "ecs_kernel_data_current_audit_static_passed_cargo_pending",
            "source files 69/69",
            "standalone `ecs_kernel_data.rs` 1/1",
            "entity/observer/command/messages/change_tick/ecs Cargo gates",
        ],
    ),
    (
        "Runtime 08 ECS source/test inventory split",
        [
            "ecs_kernel_data_source_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "ecs_kernel_data_source_inventory.py",
            "EXPECTED_SOURCE_FILE_COUNT = 69",
            "EXPECTED_TEST_FILE_COUNT = 8",
        ],
    ),
    (
        "Runtime 08 ECS anchor inventory split",
        [
            "ecs_kernel_data_anchor_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "ecs_kernel_data_anchor_inventory.py",
            "archetype_anchor_count = 15",
            "behavior_test_anchor_count = 16",
        ],
    ),
    (
        "Runtime 08 ECS markdown renderer split",
        [
            "ecs_kernel_data_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "ecs_kernel_data_markdown.py",
            "ecs_kernel_data_boundary.py` now owns audit read, missing-anchor calculation, and risk aggregation at 344 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 08 QueryState Markdown renderer split",
        [
            "ecs_query_state_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "ecs_query_state_markdown.py",
            "ecs_query_state_boundary.py` now owns QueryState owner-module audit, root budget checks, forbidden-root behavior scan, and risk aggregation at 141 lines",
            "standalone `ecs_query_structure.rs` 11/11",
        ],
    ),
    (
        "Runtime 08 F17 entity path lookup verb rename",
        [
            "runtime_08_entity_path_lookup_getter_rename_coremin_check_passed",
            "review_f17_entity_path_option_lookup_uses_get_verb",
            "get_entity_by_path",
            "old resolve-verb entity path method absent",
        ],
    ),
    (
        "Runtime 08 F5 world typed mutation errors",
        [
            "world_typed_mutation_errors_coremin_check_passed_partial",
            "SceneError::MissingEntity",
            "SceneResult",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
        ],
    ),
    (
        "Runtime 08 F5 dynamic component typed errors",
        [
            "dynamic_component_typed_errors_coremin_check_passed",
            "SceneError::PluginComponentsActive",
            "DynamicSceneError::WorldMutation(SceneError)",
            "review_f5_dynamic_component_errors_preserve_scene_error_sources",
        ],
    ),
    (
        "Runtime 08 QueryState cache owner split",
        [
            "query_state/cache.rs",
            "root_non_empty_lines = 84/180",
            "expected_module_count = 9",
            "entity/observer/command/messages/change_tick/ecs Cargo gates",
        ],
    ),
    (
        "Runtime 08 ECS event owner folder split",
        [
            "scene/ecs/events/{mod,cursor,id,metrics,queue,store,subscription}.rs",
            "EventStore::send_by_id",
            "source files 26/26",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS message owner folder split",
        [
            "scene/ecs/messages/{mod,cursor,id,queue,store}.rs",
            "MessageStore",
            "source files 30/30",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS resource store owner folder split",
        [
            "scene/ecs/resource_store/{mod,stored_resource,store}.rs",
            "ResourceStore",
            "source files 33/33",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS resource identity owner folder split",
        [
            "scene/ecs/resource/{mod,marker,id,registry}.rs",
            "ResourceRegistry",
            "source files 37/37",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS component identity owner folder split",
        [
            "scene/ecs/component/{mod,marker,id,registry}.rs",
            "ComponentRegistry",
            "source files 41/41",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS entity identity owner folder split",
        [
            "scene/ecs/entity/{mod,despawned,error,internal,location,registry,slot,stable_location}.rs",
            "EntityRegistry",
            "source files 45/45",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS archetype owner folder split",
        [
            "scene/ecs/archetype/{mod,id,index,move_result,record,signature}.rs",
            "ArchetypeIndex",
            "source files 51/51",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS component storage owner folder split",
        [
            "scene/ecs/storage/component_storage/{mod,entry,location,sparse,store,table,utils}.rs",
            "ComponentStorage",
            "source files 57/57",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS component storage private re-export cleanup",
        [
            "ecs_component_storage_private_reexport_cargo_check_passed",
            "component_storage_private_reexport_anchors = 9/9",
            "unexpected_component_storage_private_reexports = []",
            "standalone `ecs_kernel_data.rs` 1/1",
        ],
    ),
    (
        "Runtime 08 ECS observer owner folder split",
        [
            "scene/ecs/observer/{mod,callbacks,entry,id,store,utils}.rs",
            "ObserverStore",
            "source files 65/65",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS commands facade owner split",
        [
            "scene/ecs/commands/commands/{mod,entity_commands,facade,param}.rs",
            "CommandsParam",
            "source files 65/65",
            "Cargo 行为 gate",
        ],
    ),
    (
        "Runtime 08 ECS command Cargo 验证窗口探测",
        [
            "cargo test -p zircon_runtime --lib command",
            "904s timeout no result",
            "codex-runtime08-commands-owner-0620",
            "no residual cargo/rustc",
        ],
    ),
    (
        "Runtime 08 ECS entity Cargo 验证窗口探测",
        [
            "cargo test -p zircon_runtime --lib entity",
            "1200s tool window",
            "zircon-runtime08-component-storage-private-0620",
            "residual cargo/rustc processes",
        ],
    ),
    (
        "Runtime 08 ECS data owner-tree guard",
        [
            "runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover",
            "scene/ecs/{archetype,component,entity,events,messages,observer,resource,resource_store}",
            "retired flat Runtime 08 ECS owner",
            "structural module/export owner",
        ],
    ),
    (
        "Runtime 08 ECS change detection owner-tree guard",
        [
            "runtime_08_ecs_change_detection_owner_tree_stays_folder_backed_after_cutover",
            "scene/ecs/change_detection/{mod,change_tick,change_tick_window,component_ticks,stats,wrappers}.rs",
            "retired flat `scene/ecs/change_detection.rs`",
            "Cargo gate",
        ],
    ),
    (
        "Runtime 08 ECS root leaf owner guard",
        [
            "runtime_08_ecs_root_leaf_owners_stay_explicit_after_data_cutover",
            "scene/ecs/{bundle,removal,storage_type}.rs",
            "RemovedComponentEvent",
            "source files 69/69",
        ],
    ),
    (
        "Runtime 08 ecs_events_messages Cargo 验证窗口探测",
        [
            "cargo test -p zircon_runtime --lib ecs_events_messages",
            "1200s timeout no result",
            "zircon-runtime-08-ecs-events-messages-0620",
            "residual target-dir processes stopped",
        ],
    ),
];
