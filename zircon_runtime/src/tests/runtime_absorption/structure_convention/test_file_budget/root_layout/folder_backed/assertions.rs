use super::*;
use super::{guard_names::GuardNames, sources::GuardSources};

pub(super) fn assert_test_file_budget_root_is_folder_backed(
    sources: &GuardSources,
    guards: &GuardNames,
) {
    assert!(
        !sources.old_parent.exists(),
        "test-file budget guard root should live at test_file_budget/mod.rs, not the retired flat test_file_budget.rs"
    );
    assert_contains_all(
        "test file budget parent mounts folder-backed guard owners",
        &sources.parent,
        &[
            "mod asset_tests;",
            "mod asset_gltf_importer;",
            "mod asset_gltf_primitive_fixtures;",
            "mod asset_importer;",
            "mod asset_project_flow_sample;",
            "mod asset_scene;",
            "mod code_review_findings;",
            "mod core_framework;",
            "mod core_runtime_deactivation;",
            "mod dynamic_scene_absorption;",
            "mod historical_oversized_roots;",
            "mod module_layout;",
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
            "mod ui_shared_core;",
            "mod ui_v2_asset;",
        ],
    );

    for moved_guard in [
        guards.asset_pack_guard.as_str(),
        guards.asset_facade_guard.as_str(),
        guards.asset_project_zmeta_guard.as_str(),
        guards.asset_project_manager_guard.as_str(),
        guards.asset_material_guard.as_str(),
        guards.asset_gltf_importer_guard.as_str(),
        guards.asset_gltf_primitive_fixtures_guard.as_str(),
        guards.asset_importer_guard.as_str(),
        guards.asset_project_flow_sample_guard.as_str(),
        guards.asset_scene_guard.as_str(),
        guards.code_review_findings_guard.as_str(),
        guards.core_runtime_deactivation_guard.as_str(),
        guards.dynamic_scene_absorption_guard.as_str(),
        guards.render_products_guard.as_str(),
        guards.root_layout_guard.as_str(),
        guards.runtime_diagnostics_guard.as_str(),
        guards.rhi_command_list_guard.as_str(),
        guards.rhi_device_contract_guard.as_str(),
        guards.scene_component_structure_guard.as_str(),
        guards.scene_derived_state_guard.as_str(),
        guards.scene_dynamic_scene_root_guard.as_str(),
        guards.scene_dynamic_session_guard.as_str(),
        guards.scene_ecs_reflect_foundation_guard.as_str(),
        guards.scene_ecs_query_guard.as_str(),
        guards.scene_ecs_query_structure_guard.as_str(),
        guards.scene_ecs_schedule_guard.as_str(),
        guards.scene_ecs_systems_guard.as_str(),
        guards.shader_prewarm_manifest_guard.as_str(),
        guards.status_output_row_data_guard.as_str(),
        "fn runtime_15_core_framework_tests_are_folder_backed",
        "fn runtime_15_test_file_budget_parent_guard_child_owner_split",
        "fn runtime_15_ui_shared_core_tests_are_folder_backed",
        "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
        guards.script_vm_tests_guard.as_str(),
        guards.gameplay_host_tests_guard.as_str(),
    ] {
        assert!(
            !sources.parent.contains(moved_guard),
            "test_file_budget/mod.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "asset test-budget child owns child-owner mounts",
        &sources.asset_tests,
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
        &sources.asset_test_pack,
        &["use super::*;", guards.asset_pack_guard.as_str()],
    );
    assert_contains_all(
        "asset facade test-budget child owns facade guard",
        &sources.asset_test_facade,
        &["use super::*;", guards.asset_facade_guard.as_str()],
    );
    assert_contains_all(
        "asset project test-budget child owns project guards",
        &sources.asset_test_project,
        &[
            "use super::*;",
            guards.asset_project_zmeta_guard.as_str(),
            guards.asset_project_manager_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset material test-budget child owns material guard",
        &sources.asset_test_material,
        &["use super::*;", guards.asset_material_guard.as_str()],
    );
    assert_contains_all(
        "asset glTF importer test-budget child owns glTF importer guard",
        &sources.asset_gltf_importer,
        &["use super::*;", guards.asset_gltf_importer_guard.as_str()],
    );
    assert_contains_all(
        "asset glTF primitive fixture test-budget child owns fixture guard",
        &sources.asset_gltf_primitive_fixtures,
        &[
            "use super::*;",
            guards.asset_gltf_primitive_fixtures_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset importer test-budget child owns importer guard",
        &sources.asset_importer,
        &["use super::*;", guards.asset_importer_guard.as_str()],
    );
    assert_contains_all(
        "asset project flow sample test-budget child owns project flow guard",
        &sources.asset_project_flow_sample,
        &[
            "use super::*;",
            guards.asset_project_flow_sample_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset scene test-budget child owns scene guard",
        &sources.asset_scene,
        &["use super::*;", guards.asset_scene_guard.as_str()],
    );
    assert_contains_all(
        "code review findings test-budget child owns findings guard",
        &sources.code_review_findings,
        &["use super::*;", guards.code_review_findings_guard.as_str()],
    );
    assert_contains_all(
        "core framework test-budget child owns historical core-framework guard",
        &sources.core_framework,
        &[
            "use super::*;",
            "fn runtime_15_core_framework_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "core runtime deactivation test-budget child owns deactivation guard",
        &sources.core_runtime_deactivation,
        &[
            "use super::*;",
            guards.core_runtime_deactivation_guard.as_str(),
        ],
    );
    assert_contains_all(
        "dynamic-scene absorption test-budget child owns absorption guard",
        &sources.dynamic_scene_absorption,
        &[
            "use super::*;",
            guards.dynamic_scene_absorption_guard.as_str(),
        ],
    );
    assert_contains_all(
        "runtime diagnostics test-budget child owns diagnostics guard",
        &sources.runtime_diagnostics,
        &["use super::*;", guards.runtime_diagnostics_guard.as_str()],
    );
    assert_contains_all(
        "scene component-structure test-budget child owns component-structure guard",
        &sources.scene_component_structure,
        &[
            "use super::*;",
            guards.scene_component_structure_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene derived-state test-budget child owns derived-state guard",
        &sources.scene_derived_state,
        &["use super::*;", guards.scene_derived_state_guard.as_str()],
    );
    assert_contains_all(
        "dynamic-scene root test-budget child owns dynamic-scene root guard",
        &sources.scene_dynamic_scene_root,
        &[
            "use super::*;",
            guards.scene_dynamic_scene_root_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene dynamic-session test-budget child owns path-management guard",
        &sources.scene_dynamic_session,
        &["use super::*;", guards.scene_dynamic_session_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS reflect foundation test-budget child owns foundation guard",
        &sources.scene_ecs_reflect_foundation,
        &[
            "use super::*;",
            guards.scene_ecs_reflect_foundation_guard.as_str(),
        ],
    );
    assert_contains_all(
        "test-file budget module-layout child owns parent guard split",
        &sources.test_file_budget_module_layout,
        &[
            "use super::*;",
            "fn runtime_15_test_file_budget_parent_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "render product test-budget child owns camera-target guard",
        &sources.render_products,
        &["use super::*;", guards.render_products_guard.as_str()],
    );
    assert_contains_all(
        "root layout test-budget child owns root guard",
        &sources.root_layout,
        &["use super::*;", guards.root_layout_guard.as_str()],
    );
    assert_contains_all(
        "RHI command-list test-budget child owns command-list guard",
        &sources.rhi_command_list,
        &["use super::*;", guards.rhi_command_list_guard.as_str()],
    );
    assert_contains_all(
        "RHI device-contract test-budget child owns device-contract guard",
        &sources.rhi_device_contract,
        &["use super::*;", guards.rhi_device_contract_guard.as_str()],
    );
    assert_contains_all(
        "script VM test-budget child owns script VM guards",
        &sources.script_vm_tests,
        &[
            "use super::*;",
            guards.script_vm_tests_guard.as_str(),
            guards.gameplay_host_tests_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene ECS schedule test-budget child owns ECS schedule guard",
        &sources.scene_ecs_schedule,
        &["use super::*;", guards.scene_ecs_schedule_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS query test-budget child owns ECS query guard",
        &sources.scene_ecs_query,
        &["use super::*;", guards.scene_ecs_query_guard.as_str()],
    );
    assert_contains_all(
        "scene ECS query structure test-budget child owns ECS query structure guard",
        &sources.scene_ecs_query_structure,
        &[
            "use super::*;",
            guards.scene_ecs_query_structure_guard.as_str(),
        ],
    );
    assert_contains_all(
        "scene ECS systems test-budget child owns ECS systems guard",
        &sources.scene_ecs_systems,
        &["use super::*;", guards.scene_ecs_systems_guard.as_str()],
    );
    assert_contains_all(
        "shader prewarm manifest test-budget child owns manifest guard",
        &sources.shader_prewarm_manifest,
        &[
            "use super::*;",
            guards.shader_prewarm_manifest_guard.as_str(),
        ],
    );
    assert_contains_all(
        "status output row-data test-budget parent mounts child guard owners",
        &sources.status_output_row_data,
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
        &sources.status_output_row_data_runtime_15,
        &[
            "use super::*;",
            guards.status_output_row_data_guard.as_str(),
        ],
    );
    assert_contains_all(
        "UI shared core test-budget child owns historical shared-core guard",
        &sources.ui_shared_core,
        &[
            "use super::*;",
            "fn runtime_15_ui_shared_core_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "UI v2 asset test-budget child owns historical v2-asset guard",
        &sources.ui_v2_asset,
        &[
            "use super::*;",
            "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
        ],
    );
}
