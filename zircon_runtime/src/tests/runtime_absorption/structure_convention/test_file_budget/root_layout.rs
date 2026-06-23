use super::*;

#[path = "root_layout/status_scan.rs"]
mod status_scan;
#[path = "root_layout/ui_children.rs"]
mod ui_children;

#[test]
fn runtime_15_test_file_budget_guard_is_folder_backed() {
    let old_parent =
        runtime_src_path("tests/runtime_absorption/structure_convention/test_file_budget.rs");
    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/mod.rs");
    let asset_tests = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs",
    );
    let asset_test_pack = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs",
    );
    let asset_test_facade = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs",
    );
    let asset_test_project = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs",
    );
    let asset_test_material = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs",
    );
    let asset_gltf_importer = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_importer.rs",
    );
    let asset_gltf_primitive_fixtures = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_primitive_fixtures.rs",
    );
    let asset_importer = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_importer.rs",
    );
    let asset_project_flow_sample = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_project_flow_sample.rs",
    );
    let asset_scene = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_scene.rs",
    );
    let code_review_findings = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs",
    );
    let core_runtime_deactivation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs",
    );
    let dynamic_scene_absorption = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/dynamic_scene_absorption.rs",
    );
    let runtime_diagnostics = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/runtime_diagnostics.rs",
    );
    let scene_component_structure = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_component_structure.rs",
    );
    let scene_derived_state = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs",
    );
    let scene_dynamic_scene_root = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_dynamic_scene_root.rs",
    );
    let scene_dynamic_session = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_dynamic_session.rs",
    );
    let scene_ecs_reflect_foundation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_reflect_foundation.rs",
    );
    let root_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs",
    );
    let render_products = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/render_products.rs",
    );
    let rhi_command_list = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/rhi_command_list.rs",
    );
    let rhi_device_contract = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs",
    );
    let script_vm_tests = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
    );
    let scene_ecs_schedule = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_schedule.rs",
    );
    let scene_ecs_query = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query.rs",
    );
    let scene_ecs_query_structure = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query_structure.rs",
    );
    let scene_ecs_systems = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs",
    );
    let shader_prewarm_manifest = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs",
    );
    let status_output_expected_slices = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs",
    );
    let status_output_row_data = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs",
    );
    let status_output_row_data_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
    );
    assert!(
        !old_parent.exists(),
        "test-file budget guard root should live at test_file_budget/mod.rs, not the retired flat test_file_budget.rs"
    );
    assert_contains_all(
        "test file budget parent mounts folder-backed guard owners",
        &parent,
        &[
            "mod asset_tests;",
            "mod asset_gltf_importer;",
            "mod asset_gltf_primitive_fixtures;",
            "mod asset_importer;",
            "mod asset_project_flow_sample;",
            "mod asset_scene;",
            "mod code_review_findings;",
            "mod core_runtime_deactivation;",
            "mod dynamic_scene_absorption;",
            "mod historical_oversized_roots;",
            "mod render_products;",
            "mod rhi_command_list;",
            "mod rhi_device_contract;",
            "mod runtime_diagnostics;",
            "mod root_layout;",
            "mod scene_component_structure;",
            "mod scene_derived_state;",
            "mod scene_dynamic_scene_root;",
            "mod scene_dynamic_session;",
            "mod scene_ecs_reflect_foundation;",
            "mod scene_ecs_query;",
            "mod scene_ecs_query_structure;",
            "mod scene_ecs_schedule;",
            "mod scene_ecs_systems;",
            "mod shader_prewarm_manifest;",
            "mod status_output_expected_slices;",
            "mod status_output_row_data;",
            "mod script_vm_tests;",
            "mod ui_architecture;",
            "mod ui_asset;",
            "mod ui_asset_mui_web_mui_x_style;",
            "mod ui_asset_mui_web_style;",
            "mod ui_accessibility;",
            "mod ui_accessibility_widget_actions;",
            "mod ui_boundary;",
            "mod ui_component_catalog;",
            "mod ui_component_catalog_component_state;",
            "mod ui_component_catalog_component_state_keyboard;",
            "mod ui_component_catalog_material_foundation;",
            "mod ui_event_routing;",
            "mod ui_layout_slots;",
            "mod ui_material_layout;",
            "mod ui_surface_dirty_domains;",
            "mod ui_surface_frame_authority;",
            "mod ui_taffy_layout_pass;",
            "mod ui_template;",
            "mod ui_widget_text_input_keyboard;",
            "mod ui_runtime_input_reply_routes;",
            "mod ui_runtime_window_event_abi;",
            "mod ui_runtime_window_input_pump;",
            "fn runtime_15_core_framework_tests_are_folder_backed",
            "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
            "fn runtime_15_ui_shared_core_tests_are_folder_backed",
        ],
    );
    let asset_pack_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_pack_tests_are_folder_backed"
    );
    let asset_facade_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_facade_tests_are_folder_backed"
    );
    let asset_project_zmeta_guard = format!(
        "{}{}",
        "fn runtime_15_asset_project", "_zmeta_tests_are_folder_backed"
    );
    let asset_project_manager_guard = format!(
        "{}{}",
        "fn runtime_15_asset_project", "_manager_tests_are_folder_backed"
    );
    let asset_material_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_material_tests_are_folder_backed"
    );
    let asset_gltf_importer_guard = format!(
        "{}{}",
        "fn runtime_15_asset_gltf", "_importer_tests_are_folder_backed"
    );
    let asset_gltf_primitive_fixtures_guard = format!(
        "{}{}",
        "fn runtime_15_asset_gltf", "_primitive_fixtures_are_folder_backed"
    );
    let asset_importer_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_importer_tests_are_folder_backed"
    );
    let asset_project_flow_sample_guard = format!(
        "{}{}",
        "fn runtime_15_asset_project", "_flow_sample_tests_are_folder_backed"
    );
    let asset_scene_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_scene_tests_are_folder_backed"
    );
    let code_review_findings_guard = format!(
        "{}{}",
        "fn runtime_15_code_review", "_findings_tests_are_folder_backed"
    );
    let core_runtime_deactivation_guard = format!(
        "{}{}",
        "fn runtime_15_core_runtime", "_deactivation_blocked_tests_are_folder_backed"
    );
    let dynamic_scene_absorption_guard = format!(
        "{}{}",
        "fn runtime_15_dynamic_scene", "_absorption_guard_is_folder_backed"
    );
    let runtime_diagnostics_guard = format!(
        "{}{}",
        "fn runtime_15_runtime", "_diagnostics_tests_are_folder_backed"
    );
    let root_layout_guard = format!(
        "{}{}",
        "fn runtime_15_test_file_budget", "_guard_is_folder_backed"
    );
    let render_products_guard = format!(
        "{}{}",
        "fn runtime_15_render_camera", "_target_products_are_folder_backed"
    );
    let rhi_command_list_guard = format!(
        "{}{}",
        "fn runtime_15_rhi", "_command_list_tests_are_folder_backed"
    );
    let rhi_device_contract_guard = format!(
        "{}{}",
        "fn runtime_15_rhi", "_device_contract_tests_are_folder_backed"
    );
    let script_vm_tests_guard = format!(
        "{}{}",
        "fn runtime_15_script", "_vm_tests_are_folder_backed"
    );
    let gameplay_host_tests_guard = format!(
        "{}{}",
        "fn runtime_15_gameplay", "_host_tests_are_folder_backed"
    );
    let scene_ecs_schedule_guard = format!(
        "{}{}",
        "fn runtime_15_scene", "_ecs_schedule_tests_are_folder_backed"
    );
    let scene_ecs_query_guard = format!(
        "{}{}",
        "fn runtime_15_scene", "_ecs_query_tests_are_folder_backed"
    );
    let scene_ecs_query_structure_guard = format!(
        "{}{}",
        "fn runtime_15_scene", "_ecs_query_structure_tests_are_folder_backed"
    );
    let scene_derived_state_guard = format!(
        "{}{}",
        "fn runtime_15_scene", "_derived_state_tests_are_folder_backed"
    );
    let scene_component_structure_guard = format!(
        "{}{}",
        "fn runtime_15_scene", "_component_structure_tests_are_folder_backed"
    );
    let scene_dynamic_scene_root_guard = format!(
        "{}{}",
        "fn runtime_15_dynamic_scene", "_root_tests_are_folder_backed"
    );
    let scene_dynamic_session_guard = format!(
        "{}{}",
        "fn runtime_15_dynamic_scene_session", "_path_management_tests_are_folder_backed"
    );
    let scene_ecs_reflect_foundation_guard = format!(
        "{}{}",
        "fn runtime_15_scene_ecs", "_reflect_foundation_tests_are_folder_backed"
    );
    let scene_ecs_systems_guard = format!(
        "{}{}",
        "fn runtime_15_scene", "_ecs_systems_tests_are_folder_backed"
    );
    let shader_prewarm_manifest_guard = format!(
        "{}{}",
        "fn runtime_15_shader_prewarm", "_manifest_tests_are_folder_backed"
    );
    let status_output_row_data_guard = format!(
        "{}{}",
        "fn runtime_15_status_output_runtime_15", "_row_data_is_child_owner"
    );
    for moved_guard in [
        asset_pack_guard.as_str(),
        asset_facade_guard.as_str(),
        asset_project_zmeta_guard.as_str(),
        asset_project_manager_guard.as_str(),
        asset_material_guard.as_str(),
        asset_gltf_importer_guard.as_str(),
        asset_gltf_primitive_fixtures_guard.as_str(),
        asset_importer_guard.as_str(),
        asset_project_flow_sample_guard.as_str(),
        asset_scene_guard.as_str(),
        code_review_findings_guard.as_str(),
        core_runtime_deactivation_guard.as_str(),
        dynamic_scene_absorption_guard.as_str(),
        render_products_guard.as_str(),
        root_layout_guard.as_str(),
        runtime_diagnostics_guard.as_str(),
        rhi_command_list_guard.as_str(),
        rhi_device_contract_guard.as_str(),
        scene_component_structure_guard.as_str(),
        scene_derived_state_guard.as_str(),
        scene_dynamic_scene_root_guard.as_str(),
        scene_dynamic_session_guard.as_str(),
        scene_ecs_reflect_foundation_guard.as_str(),
        scene_ecs_query_guard.as_str(),
        scene_ecs_query_structure_guard.as_str(),
        scene_ecs_schedule_guard.as_str(),
        scene_ecs_systems_guard.as_str(),
        shader_prewarm_manifest_guard.as_str(),
        status_output_row_data_guard.as_str(),
        script_vm_tests_guard.as_str(),
        gameplay_host_tests_guard.as_str(),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "test_file_budget/mod.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }
    assert_contains_all(
        "asset test-budget child owns child-owner mounts",
        &asset_tests,
        &[
            "use super::*;",
            "mod facade;",
            "mod material;",
            "mod pack;",
            "mod project;",
            "fn runtime_15_asset_test_budget_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "asset pack test-budget child owns pack guard",
        &asset_test_pack,
        &["use super::*;", asset_pack_guard.as_str()],
    );
    assert_contains_all(
        "asset facade test-budget child owns facade guard",
        &asset_test_facade,
        &["use super::*;", asset_facade_guard.as_str()],
    );
    assert_contains_all(
        "asset project test-budget child owns project guards",
        &asset_test_project,
        &[
            "use super::*;",
            asset_project_zmeta_guard.as_str(),
            asset_project_manager_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset material test-budget child owns material guard",
        &asset_test_material,
        &["use super::*;", asset_material_guard.as_str()],
    );
    assert_contains_all(
        "asset glTF importer test-budget child owns glTF importer guard",
        &asset_gltf_importer,
        &["use super::*;", asset_gltf_importer_guard.as_str()],
    );
    assert_contains_all(
        "asset glTF primitive fixture test-budget child owns fixture guard",
        &asset_gltf_primitive_fixtures,
        &[
            "use super::*;",
            asset_gltf_primitive_fixtures_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset importer test-budget child owns importer guard",
        &asset_importer,
        &["use super::*;", asset_importer_guard.as_str()],
    );
    assert_contains_all(
        "asset project flow sample test-budget child owns project flow guard",
        &asset_project_flow_sample,
        &["use super::*;", asset_project_flow_sample_guard.as_str()],
    );
    assert_contains_all(
        "asset scene test-budget child owns scene guard",
        &asset_scene,
        &["use super::*;", asset_scene_guard.as_str()],
    );
    assert_contains_all(
        "code review findings test-budget child owns findings guard",
        &code_review_findings,
        &["use super::*;", code_review_findings_guard.as_str()],
    );
    assert_contains_all(
        "core runtime deactivation test-budget child owns deactivation guard",
        &core_runtime_deactivation,
        &["use super::*;", core_runtime_deactivation_guard.as_str()],
    );
    assert_contains_all(
        "dynamic-scene absorption test-budget child owns absorption guard",
        &dynamic_scene_absorption,
        &["use super::*;", dynamic_scene_absorption_guard.as_str()],
    );
    assert_contains_all(
        "runtime diagnostics test-budget child owns diagnostics guard",
        &runtime_diagnostics,
        &["use super::*;", runtime_diagnostics_guard.as_str()],
    );
    assert_contains_all(
        "scene component-structure test-budget child owns component-structure guard",
        &scene_component_structure,
        &["use super::*;", scene_component_structure_guard.as_str()],
    );
    assert_contains_all(
        "scene derived-state test-budget child owns derived-state guard",
        &scene_derived_state,
        &["use super::*;", scene_derived_state_guard.as_str()],
    );
    assert_contains_all(
        "dynamic-scene root test-budget child owns dynamic-scene root guard",
        &scene_dynamic_scene_root,
        &["use super::*;", scene_dynamic_scene_root_guard.as_str()],
    );
    assert_contains_all(
        "scene dynamic-session test-budget child owns path-management guard",
        &scene_dynamic_session,
        &["use super::*;", scene_dynamic_session_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS reflect foundation test-budget child owns foundation guard",
        &scene_ecs_reflect_foundation,
        &["use super::*;", scene_ecs_reflect_foundation_guard.as_str()],
    );
    assert_contains_all(
        "render product test-budget child owns camera-target guard",
        &render_products,
        &["use super::*;", render_products_guard.as_str()],
    );
    assert_contains_all(
        "root layout test-budget child owns root guard",
        &root_layout,
        &["use super::*;", root_layout_guard.as_str()],
    );
    assert_contains_all(
        "RHI command-list test-budget child owns command-list guard",
        &rhi_command_list,
        &["use super::*;", rhi_command_list_guard.as_str()],
    );
    assert_contains_all(
        "RHI device-contract test-budget child owns device-contract guard",
        &rhi_device_contract,
        &["use super::*;", rhi_device_contract_guard.as_str()],
    );
    assert_contains_all(
        "script VM test-budget child owns script VM guards",
        &script_vm_tests,
        &[
            "use super::*;",
            script_vm_tests_guard.as_str(),
            gameplay_host_tests_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene ECS schedule test-budget child owns ECS schedule guard",
        &scene_ecs_schedule,
        &["use super::*;", scene_ecs_schedule_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS query test-budget child owns ECS query guard",
        &scene_ecs_query,
        &["use super::*;", scene_ecs_query_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS query structure test-budget child owns ECS query structure guard",
        &scene_ecs_query_structure,
        &["use super::*;", scene_ecs_query_structure_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS systems test-budget child owns ECS systems guard",
        &scene_ecs_systems,
        &["use super::*;", scene_ecs_systems_guard.as_str()],
    );
    assert_contains_all(
        "shader prewarm manifest test-budget child owns manifest guard",
        &shader_prewarm_manifest,
        &["use super::*;", shader_prewarm_manifest_guard.as_str()],
    );
    assert_contains_all(
        "status output row-data test-budget parent mounts child guard owners",
        &status_output_row_data,
        &[
            "use super::*;",
            "#[path = \"status_output_row_data/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"status_output_row_data/runtime_15_row_data.rs\"]",
            "mod runtime_15_row_data;",
        ],
    );
    assert_contains_all(
        "status output row-data Runtime 15 child owns Runtime 15 row-data guard",
        &status_output_row_data_runtime_15,
        &["use super::*;", status_output_row_data_guard.as_str()],
    );
}
