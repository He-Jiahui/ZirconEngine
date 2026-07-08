use super::super::*;
use super::super::{guard_names::GuardNames, sources::GuardSources};

pub(super) fn assert_render_status_children(sources: &GuardSources, guards: &GuardNames) {
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
            "#[path = \"shader_prewarm_manifest/manifest_contract.rs\"]",
            "mod manifest_contract;",
            "fn runtime_15_shader_prewarm_manifest_guard_children_are_folder_backed",
        ],
    );
    assert_contains_all(
        "status output expected-slices test-budget parent mounts child guard owners",
        &sources.status_output_expected_slices,
        &[
            "use super::*;",
            "#[path = \"status_slices/legacy_group_maps.rs\"]",
            "mod legacy_group_maps;",
            "#[path = \"status_slices/maps.rs\"]",
            "mod maps;",
            "#[path = \"status_slices/module_layout.rs\"]",
            "mod module_layout;",
        ],
    );
    assert_contains_all(
        "status output row-data test-budget parent mounts child guard owners",
        &sources.status_output_row_data,
        &[
            "use super::*;",
            "#[path = \"row_data/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"row_data/runtime_15_row_data.rs\"]",
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
}
