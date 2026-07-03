use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
const ROOT_PARENT_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs";
const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) fn assert_code_review_root_parent_is_folder_backed(sources: &CodeReviewFindingsSources) {
    assert_contains_all(
        "code review findings parent mounts folder-backed children",
        &sources.parent,
        &[
            "mod f12_dead_code;",
            "mod f8_api_convergence;",
            "mod late_api_cleanup;",
            "mod p0_robustness;",
            "mod plugin_importer_dx;",
            "mod render_structure;",
            "mod typed_error_convergence;",
        ],
    );
    assert_eq!(
        sources.parent.matches("#[test]").count(),
        0,
        "code_review_findings.rs should only mount child test owners"
    );
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
fn runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD);
    let sources = super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates root parent checks to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/root_parent.rs\"]",
            "mod root_parent;",
            "root_parent::assert_code_review_root_parent_is_folder_backed",
        ],
    );
    for moved_guard in [
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
            !parent.contains(moved_guard),
            "root-parent direct assertion `{moved_guard}` should stay in {ROOT_PARENT_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "root-parent direct assertion child owns parent mount and backflow checks",
        &child,
        &[
            "pub(super) fn assert_code_review_root_parent_is_folder_backed",
            "code review findings parent mounts folder-backed children",
            "code_review_findings.rs should only mount child test owners",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_d13_importer_runtime_manifests_use_sdk_builder",
            "review_f19_scene_renderer_construction_modules_use_construct_names",
        ],
    );

    assert_code_review_root_parent_is_folder_backed(&sources);

    for (path, source) in [
        (DIRECT_REVIEW_ASSERTIONS_CHILD, parent.as_str()),
        (ROOT_PARENT_DIRECT_ASSERTIONS_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
