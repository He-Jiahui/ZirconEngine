use super::*;

#[test]
fn runtime_05_dynamic_scene_patch_preview_status_docs_stay_synced() {
    for documented_source in [RUNTIME_05_PLAN, RUNTIME_INDEX] {
        for anchor in [
            "Runtime 05 dynamic scene patch preview API",
            "dynamic_scene_patch_preview_api_static_passed_cargo_timeout_no_result_tests_deferred",
            "Runtime 05 dynamic scene patch preview status guard",
            "dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending",
            "ScenePatchPreviewReport",
            "ScenePatch::preview_apply",
            "DynamicScene::preview_spawn_into",
            "scene_patch_preview_reports_remaps_without_mutating_target_world",
            "Runtime 05 dynamic scene patch preview resource preflight details status guard",
            "dynamic_scene_patch_preview_resource_preflight_details_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_resource_preflight_details_static_passed_cargo_deferred_tests_deferred",
            "ScenePatchPreviewResource",
            "resources_requiring_creation()",
            "already_present",
            "can_create_on_apply",
            "Runtime 05 dynamic scene patch preview resource ensure creation status guard",
            "dynamic_scene_patch_preview_resource_ensure_creation_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_resource_ensure_creation_static_passed_cargo_deferred_tests_deferred",
            "register_frame_counter_resource_with_ensure",
            "frame_counter_adapter_with_ensure",
            "frame_counter_ensure",
            "preview_with_ensure.resources[0].can_create_on_apply",
            "Runtime 05 dynamic scene patch preview component type install details status guard",
            "dynamic_scene_patch_preview_component_type_install_details_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_component_type_install_details_static_passed_cargo_deferred_tests_deferred",
            "ScenePatchPreviewComponentType",
            "component_types",
            "already_registered",
            "Runtime 05 dynamic scene patch preview component type install counts status guard",
            "dynamic_scene_patch_preview_component_type_install_counts_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_component_type_install_counts_static_passed_cargo_deferred_tests_deferred",
            "existing_component_type_count",
            "new_component_type_count",
            "has_new_component_types()",
            "Runtime 05 dynamic scene patch preview reflection preflight status guard",
            "dynamic_scene_patch_preview_reflection_preflight_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_reflection_preflight_static_passed_cargo_deferred_tests_deferred",
            "validate_components_are_previewable",
            "ReflectError::MissingResource",
            "Runtime 05 dynamic scene patch preview component workload status guard",
            "dynamic_scene_patch_preview_component_workload_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_component_workload_static_passed_cargo_deferred_tests_deferred",
            "component_instance_count",
            "Runtime 05 dynamic scene patch preview remap status guard",
            "dynamic_scene_patch_preview_remap_status_guard_static_passed_cargo_pending",
            "ScenePatchPreviewEntityRemap",
            "entity_remaps",
            "has_entity_remaps()",
        ] {
            assert!(
                documented_source.contains(anchor),
                "Runtime 05 patch preview status/doc source should keep anchor `{anchor}`"
            );
        }
    }
    for anchor in [
        "dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending",
        "Runtime 05 dynamic scene patch preview status guard",
        "dynamic_scene_patch_preview_remap_details_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_component_workload_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_reflection_preflight_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_component_type_install_counts_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_component_type_install_details_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_resource_preflight_details_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_resource_preflight_details_status_guard_static_passed_cargo_pending",
        "Runtime 05 dynamic scene patch preview resource preflight details status guard",
        "dynamic_scene_patch_preview_resource_ensure_creation_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_resource_ensure_creation_status_guard_static_passed_cargo_pending",
        "Runtime 05 dynamic scene patch preview resource ensure creation status guard",
        "ScenePatchPreviewComponentType",
        "ScenePatchPreviewResource",
        "ScenePatchPreviewEntityRemap",
        "component_types",
        "already_registered",
        "already_present",
        "can_create_on_apply",
        "new_component_type_count",
        "resources_requiring_creation()",
        "component_instance_count",
    ] {
        assert!(
            RUNTIME_05_PLAN.contains(anchor),
            "Runtime 05 patch preview subplan should keep localized anchor `{anchor}`"
        );
    }
    for anchor in [
        "Dynamic scene patch preview API",
        "ScenePatchPreviewReport",
        "ScenePatchPreviewEntityRemap",
        "ScenePatchPreviewComponentType",
        "ScenePatchPreviewResource",
        "already_registered",
        "already_present",
        "can_create_on_apply",
        "new_component_type_count",
        "resource preflight details",
        "ensure-backed resource",
        "component type install preview",
        "component_instance_count",
        "reflection schema preflight",
        "ScenePatch::preview_apply",
        "DynamicScene::preview_spawn_into",
        "scene_patch_preview_reports_remaps_without_mutating_target_world",
    ] {
        assert!(
            DYNAMIC_SCENE_DOC.contains(anchor),
            "dynamic-scene module docs should keep behavior anchor `{anchor}`"
        );
    }
}
