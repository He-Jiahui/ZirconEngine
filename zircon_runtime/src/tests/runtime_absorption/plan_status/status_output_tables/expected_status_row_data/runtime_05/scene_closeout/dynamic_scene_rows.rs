use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 dynamic scene patch preview API",
        &[
            "ScenePatchPreviewReport",
            "ScenePatch::preview_apply",
            "DynamicScene::preview_spawn_into",
            "scene_patch_preview_reports_remaps_without_mutating_target_world",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview status guard",
        &[
            "runtime_05_dynamic_scene_patch_preview_api_stays_read_only",
            "ScenePatchPreviewReport",
            "ScenePatch::preview_apply",
            "scene_patch_preview_reports_remaps_without_mutating_target_world",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview resource preflight details status guard",
        &[
            "ScenePatchPreviewResource",
            "resources_requiring_creation()",
            "already_present",
            "dynamic_scene_patch_preview_resource_preflight_details_static_passed_cargo_deferred_tests_deferred",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview resource ensure creation status guard",
        &[
            "register_frame_counter_resource_with_ensure",
            "frame_counter_adapter_with_ensure",
            "preview_with_ensure.resources[0].can_create_on_apply",
            "dynamic_scene_patch_preview_resource_ensure_creation_static_passed_cargo_deferred_tests_deferred",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview component type install details status guard",
        &[
            "ScenePatchPreviewComponentType",
            "component_types",
            "already_registered",
            "dynamic_scene_patch_preview_component_type_install_details_static_passed_cargo_deferred_tests_deferred",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview component type install counts status guard",
        &[
            "existing_component_type_count",
            "new_component_type_count",
            "has_new_component_types()",
            "dynamic_scene_patch_preview_component_type_install_counts_static_passed_cargo_deferred_tests_deferred",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview reflection preflight status guard",
        &[
            "validate_components_are_previewable",
            "validate_resources_are_previewable",
            "ReflectError::MissingResource",
            "dynamic_scene_patch_preview_reflection_preflight_static_passed_cargo_deferred_tests_deferred",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview component workload status guard",
        &[
            "runtime_05_dynamic_scene_patch_preview_api_stays_read_only",
            "component_instance_count",
            "preview.component_instance_count",
            "dynamic_scene_patch_preview_component_workload_static_passed_cargo_deferred_tests_deferred",
        ],
    ),
    (
        "Runtime 05 dynamic scene patch preview remap status guard",
        &[
            "runtime_05_dynamic_scene_patch_preview_api_stays_read_only",
            "ScenePatchPreviewEntityRemap",
            "entity_remaps",
            "has_entity_remaps()",
        ],
    ),
    (
        "Runtime 05 dynamic scene root scene owner split",
        &[
            "scene/{mod,capture,spawn,validation}.rs",
            "DynamicScene::from_world",
            "DynamicScene::spawn_into",
            "standalone plan-status 32/32",
        ],
    ),
    (
        "Runtime 05 dynamic scene document serialization owner split",
        &[
            "document/{mod,legacy,read,write}.rs",
            "DynamicScene::from_versioned_json",
            "DynamicScene::to_versioned_json_pretty",
            "legacy `ProjectDocument { world }` migration",
        ],
    ),
    (
        "Runtime 05 dynamic scene entity declaration owner split",
        &[
            "entity/{mod,dynamic_component,dynamic_entity,dynamic_resource}.rs",
            "DynamicComponent",
            "DynamicEntity",
            "serialization source guard",
        ],
    ),
    (
        "Runtime 05 dynamic scene scene-asset bridge owner split",
        &[
            "scene_asset/{mod,dynamic_scene,error,prepared_spawn}.rs",
            "DynamicScene::from_scene_asset",
            "PreparedDynamicSceneSpawn::from_scene_asset",
            "SceneAssetSerializer",
        ],
    ),
    (
        "Runtime 05 dynamic scene spawn task owner split",
        &[
            "spawn_task/{mod,loader,prepared,task}.rs",
            "PreparedDynamicSceneSpawn",
            "DynamicSceneSpawnTask",
            "standalone plan-status 32/32",
        ],
    ),
    (
        "Runtime 05 dynamic scene value conversion owner split",
        &[
            "value/{mod,json,remap}.rs",
            "reflected fields -> JSON object conversion",
            "JSON `{ \"entity\": ... }` remap",
            "standalone plan-status 32/32",
        ],
    ),
    (
        "Runtime 05 dynamic scene session owner-tree guard",
        &[
            "dynamic_scene_session_owner_tree_stays_folder_backed_after_runtime_05_cutover",
            "session/{metadata,manifest,query,reports,merge,retention,slot_copy,slot_export,slot_import,slot_mutation,capture_retention}",
            "retired flat session owner files stay absent",
            "session/mod.rs remains structural",
        ],
    ),
    (
        "Runtime 05 dynamic scene root owner-tree guard",
        &[
            "dynamic_scene_root_owner_tree_stays_folder_backed_after_runtime_05_cutover",
            "document/entity/scene/scene_asset/spawn_task/value",
            "retired flat dynamic_scene owner files stay absent",
            "scene/mod.rs keeps DynamicScene facade",
        ],
    ),
];
