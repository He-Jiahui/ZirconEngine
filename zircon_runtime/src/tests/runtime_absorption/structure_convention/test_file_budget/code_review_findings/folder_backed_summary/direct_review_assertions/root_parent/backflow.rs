use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_code_review_root_parent_moved_tests_do_not_backflow(
    sources: &CodeReviewFindingsSources,
) {
    for moved_test in [
        "review_f5_world_spawn_bundle_surface_uses_scene_error",
        "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        "review_f2_scene_eventbus_locks_recover_after_poison",
        "review_f4_render_submit_capability_gaps_return_typed_errors",
        "review_f8_texture_import_settings_use_fallible_apply_not_with",
        "review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
        "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
        "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
        "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
        "review_d12_runtime_helper_exports_use_sdk_macro",
        "review_d5_editor_authoring_plugins_use_sdk_macro",
        "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
        "review_d13_importer_runtime_exports_use_sdk_macro",
        "review_d13_importer_runtime_manifests_use_sdk_builder",
        "review_d11_animation_physics_tests_use_sdk_test_runtime_fixture",
        "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
        "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
        "review_f17_entity_path_option_lookup_uses_get_verb",
        "review_f18_asset_manager_resolution_returns_registered_handle",
        "review_f16_compiled_scene_render_path_uses_split_owners",
        "review_f19_scene_renderer_construction_modules_use_construct_names",
    ] {
        assert!(
            !sources.parent.contains(moved_test),
            "moved code review findings test `{moved_test}` should not return to the parent"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_root_parent_direct_assertions_guard_is_folder_backed() {
    let root_parent = read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD);
    let child_blob = root_parent_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    parent_mounts::assert_code_review_root_parent_mounts_are_folder_backed(&sources);
    assert_code_review_root_parent_moved_tests_do_not_backflow(&sources);
    budgets::assert_root_parent_direct_assertions_children_line_budgets_are_current();
    for (_, child_path, child_guard) in ROOT_PARENT_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            root_parent.contains(child_path),
            "root-parent direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "root-parent direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    for parent_owned_guard in [
        concat!(
            "code review findings parent mounts ",
            "folder-backed children"
        ),
        "code_review_findings.rs should only mount child test owners",
        concat!("review_f5_world_spawn_bundle_surface_uses_", "scene_error"),
        concat!("review_d13_importer_runtime_manifests_use_", "sdk_builder"),
        concat!(
            "review_f19_scene_renderer_construction_modules_",
            "use_construct_names"
        ),
    ] {
        assert!(
            !root_parent.contains(parent_owned_guard),
            "root-parent direct assertion `{parent_owned_guard}` should stay in focused children"
        );
    }
    assert_contains_all(
        "root-parent direct assertions parent records folder-backed status",
        &root_parent,
        &[
            ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
            ROOT_PARENT_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            ROOT_PARENT_DIRECT_ASSERTIONS_STATUS_GUARD,
            ROOT_PARENT_DIRECT_ASSERTIONS_BUDGET_GUARD,
        ],
    );
}
