use super::*;

#[test]
fn runtime_15_test_file_budget_root_layout_status_scan_is_child_owner() {
    let root_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs",
    );
    let status_scan = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n");

    assert_contains_all(
        "root-layout guard parent mounts status scan child",
        &root_layout,
        &[
            "#[path = \"root_layout/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"root_layout/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"root_layout/status_scan.rs\"]",
            "mod status_scan;",
        ],
    );
    for moved_anchor in [
        "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
        "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
        "Runtime 15 M3 status output Runtime 15 row data split",
        "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !root_layout.contains(moved_anchor),
            "test_file_budget/root_layout.rs should delegate status scan anchors instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "root-layout status scan child owns status anchors",
        &status_scan,
        &[
            "Runtime 15 M3 test file budget root-layout folder-backed guard child split",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
            "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split",
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
            "Runtime 15 M3 test file budget guard folder split",
            "runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked",
            "Runtime 15 M3 test file budget root-layout child split",
            "runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred",
            "Runtime 15 M3 test file budget root-layout status scan child split",
            "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
            "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
        ],
    );

    for path in [
        "tests/runtime_absorption/structure_convention/test_file_budget/mod.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/core_framework.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/module_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/picking.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_v2_asset.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_importer.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_primitive_fixtures.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_importer.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_artifact_store.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_mesh.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_pipeline_manager.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_project_example_vampire.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_project_flow_sample.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_scene.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_ui.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/dynamic_scene_absorption.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/historical_oversized_roots.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/runtime_diagnostics.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_component_structure.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_dynamic_scene_root.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_dynamic_session.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_reflect_foundation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/render_products.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/sources.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/module_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/ui_children.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/rhi_command_list.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_architecture.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset_mui_web_form_style.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset_mui_web_mui_x_style.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset_mui_web_style.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_boundary.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility_widget_actions.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state_keyboard.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_material_foundation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_event_routing.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_focus_navigation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_layout_slots.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_material_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_dirty_domains.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_frame_authority.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_taffy_layout_pass.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_template.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_widget_text_input_keyboard.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_ownership.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_event_abi.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_input_pump.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_schedule.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query_structure.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 test file budget guard folder split",
                "runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked",
                "Runtime 15 M3 test file budget guard root mod cutover",
                "runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked",
                "structure_convention/test_file_budget/mod.rs",
                "structure_convention/test_file_budget/runtime_diagnostics.rs",
                "structure_convention/test_file_budget/rhi_device_contract.rs",
                "structure_convention/test_file_budget/asset_gltf_primitive_fixtures.rs",
                "structure_convention/test_file_budget/asset_project_flow_sample.rs",
                "structure_convention/test_file_budget/asset_scene.rs",
                "structure_convention/test_file_budget/script_vm_tests.rs",
                "runtime_15_test_file_budget_guard_is_folder_backed",
                "Runtime 15 M3 test file budget root-layout child split",
                "runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/root_layout.rs",
                "Runtime 15 M3 asset test-budget guard child-owner split",
                "runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/asset_tests/pack.rs",
                "structure_convention/test_file_budget/asset_tests/project.rs",
                "Runtime 15 M3 gameplay host test folder split",
                "runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred",
                "script/vm/gameplay_host/tests/spawn_transform.rs",
                "script/vm/gameplay_host/tests/property_animation.rs",
                "runtime_15_gameplay_host_tests_are_folder_backed",
                "Runtime 15 M3 shader prewarm manifest test folder split",
                "runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred",
                "bin/zircon_shader_prewarm/manifest.rs",
                "bin/zircon_shader_prewarm/manifest/tests.rs",
                "runtime_15_shader_prewarm_manifest_tests_are_folder_backed",
                "Runtime 15 M3 status output Runtime 15 row data split",
                "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
                "runtime_15_status_output_runtime_15_row_data_is_child_owner",
                "Runtime 15 M3 test file budget root-layout status scan child split",
                "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/root_layout/status_scan.rs",
                "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
                "Runtime 15 M3 test file budget root-layout folder-backed guard child split",
                "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
                "structure_convention/test_file_budget/root_layout/folder_backed.rs",
                "structure_convention/test_file_budget/root_layout/module_layout.rs",
                "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
                "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split",
                "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
                "structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
                "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
                "Runtime 15 M3 test file budget parent guard child-owner split",
                "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/core_framework.rs",
                "structure_convention/test_file_budget/ui_v2_asset.rs",
                "structure_convention/test_file_budget/ui_shared_core.rs",
                "structure_convention/test_file_budget/module_layout.rs",
                "runtime_15_test_file_budget_parent_guard_child_owner_split",
            ],
        );
    }
}
