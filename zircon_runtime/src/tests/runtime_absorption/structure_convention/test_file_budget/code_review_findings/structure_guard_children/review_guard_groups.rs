use super::super::super::*;
use super::*;

pub(super) fn assert_review_guard_group_children_are_mounted() {
    let f8_child = format!(
        "{}\n{}",
        read_runtime_src(F8_CHILD_OWNER),
        super::super::f8_child_owners::f8_structure_guard_child_source_blob(),
    );
    let late_api_cleanup_child =
        super::super::late_api_cleanup_child_owners::late_api_cleanup_structure_guard_child_source_blob();
    let p0_child = super::super::p0_child_owners::p0_structure_guard_child_source_blob();
    let p0_native_fixture_leaf_child =
        super::super::p0_native_fixture_leaf_owners::p0_native_fixture_structure_guard_child_source_blob();

    assert_contains_all(
        "F8 structure child owner keeps F8 review guard ownership checks",
        &f8_child,
        &[
            "fn runtime_15_f8_api_convergence_review_guards_are_child_owners",
            "Runtime 15 M3 F8 child-owner structure guard folder-backed split",
            "runtime_15_f8_child_owner_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_f8_child_owner_structure_guard_is_folder_backed",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/texture_import_settings.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/first_party_descriptors.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/private_fields.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/constructor_retirement.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/status_mirrors.rs",
            "review_f8_texture_import_settings_use_fallible_apply_not_with",
        ],
    );
    assert_contains_all(
        "late API cleanup structure child owner keeps late API review guard ownership checks",
        &late_api_cleanup_child,
        &[
            "fn runtime_15_late_api_cleanup_review_guards_are_child_owners",
            "Runtime 15 M3 late API cleanup structure guard folder-backed split",
            "runtime_15_late_api_cleanup_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_late_api_cleanup_structure_guard_is_folder_backed",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup/f11_shading_model_registry.rs",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup/f15_editor_pane_data_conversion.rs",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup/f17_entity_path_lookup.rs",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup/f18_asset_manager_resolution.rs",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup/f19_scene_renderer_construction.rs",
            "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
            "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
            "review_f17_entity_path_option_lookup_uses_get_verb",
            "review_f18_asset_manager_resolution_returns_registered_handle",
            "review_f19_scene_renderer_construction_modules_use_construct_names",
        ],
    );
    assert_contains_all(
        "P0 structure child owner keeps P0 review guard ownership checks",
        &p0_child,
        &[
            "fn runtime_15_p0_robustness_review_guards_are_child_owners",
            "Runtime 15 M3 P0 robustness structure guard folder-backed split",
            "runtime_15_p0_robustness_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_p0_robustness_structure_guard_is_folder_backed",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/delegation.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/route_ownership.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/status_mirrors.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/budgets.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_host_callbacks.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/lock_poison.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/render_submit.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/importer_manifest.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs",
            "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
            "review_f4_render_submit_capability_gaps_return_typed_errors",
        ],
    );
    assert_contains_all(
        "P0 native fixture leaf structure child owner keeps native fixture leaf checks",
        &p0_native_fixture_leaf_child,
        &[
            "fn runtime_15_p0_native_fixture_review_guards_are_leaf_owners",
            "Runtime 15 M3 P0 native fixture review guard leaf-owner split",
            "runtime_15_p0_native_fixture_review_guard_leaf_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split",
            "runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_p0_native_fixture_leaf_owner_guard_is_folder_backed",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/importer_manifest.rs",
            "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
            "review_d13_native_fixture_importer_is_manifest_described",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_review_groups_are_child_owned() {
    assert_review_guard_group_children_are_mounted();
}
