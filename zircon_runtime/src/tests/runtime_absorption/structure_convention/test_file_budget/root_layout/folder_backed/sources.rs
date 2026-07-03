use super::*;
use std::path::PathBuf;

pub(super) struct GuardSources {
    pub(super) old_parent: PathBuf,
    pub(super) parent: String,
    pub(super) asset_tests: String,
    pub(super) asset_test_pack: String,
    pub(super) asset_test_facade: String,
    pub(super) asset_test_project: String,
    pub(super) asset_test_material: String,
    pub(super) asset_gltf_importer: String,
    pub(super) asset_gltf_primitive_fixtures: String,
    pub(super) asset_importer: String,
    pub(super) asset_project_flow_sample: String,
    pub(super) asset_scene: String,
    pub(super) code_review_findings: String,
    pub(super) core_framework: String,
    pub(super) core_runtime_deactivation: String,
    pub(super) dynamic_scene_absorption: String,
    pub(super) runtime_diagnostics: String,
    pub(super) scene_component_structure: String,
    pub(super) scene_derived_state: String,
    pub(super) scene_dynamic_scene_root: String,
    pub(super) scene_dynamic_session: String,
    pub(super) scene_ecs_reflect_foundation: String,
    pub(super) test_file_budget_module_layout: String,
    pub(super) root_layout: String,
    pub(super) render_products: String,
    pub(super) rhi_command_list: String,
    pub(super) rhi_device_contract: String,
    pub(super) script_vm_tests: String,
    pub(super) scene_ecs_schedule: String,
    pub(super) scene_ecs_query: String,
    pub(super) scene_ecs_query_structure: String,
    pub(super) scene_ecs_systems: String,
    pub(super) shader_prewarm_manifest: String,
    pub(super) status_output_expected_slices: String,
    pub(super) status_output_row_data: String,
    pub(super) status_output_row_data_runtime_15: String,
    pub(super) ui_shared_core: String,
    pub(super) ui_v2_asset: String,
}

pub(super) fn read_guard_sources() -> GuardSources {
    GuardSources {
        old_parent: runtime_src_path(
            "tests/runtime_absorption/structure_convention/test_file_budget.rs",
        ),
        parent: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/mod.rs",
        ),
        asset_tests: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs",
        ),
        asset_test_pack: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs",
        ),
        asset_test_facade: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs",
        ),
        asset_test_project: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs",
        ),
        asset_test_material: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs",
        ),
        asset_gltf_importer: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_importer.rs",
        ),
        asset_gltf_primitive_fixtures: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_primitive_fixtures.rs",
        ),
        asset_importer: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_importer.rs",
        ),
        asset_project_flow_sample: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_project_flow_sample.rs",
        ),
        asset_scene: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_scene.rs",
        ),
        code_review_findings: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs",
        ),
        core_framework: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/core_framework.rs",
        ),
        core_runtime_deactivation: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs",
        ),
        dynamic_scene_absorption: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/dynamic_scene_absorption.rs",
        ),
        runtime_diagnostics: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/runtime_diagnostics.rs",
        ),
        scene_component_structure: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_component_structure.rs",
        ),
        scene_derived_state: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs",
        ),
        scene_dynamic_scene_root: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_dynamic_scene_root.rs",
        ),
        scene_dynamic_session: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_dynamic_session.rs",
        ),
        scene_ecs_reflect_foundation: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_reflect_foundation.rs",
        ),
        test_file_budget_module_layout: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/module_layout.rs",
        ),
        root_layout: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
        ),
        render_products: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/render_products.rs",
        ),
        rhi_command_list: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/rhi_command_list.rs",
        ),
        rhi_device_contract: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs",
        ),
        script_vm_tests: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
        ),
        scene_ecs_schedule: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_schedule.rs",
        ),
        scene_ecs_query: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query.rs",
        ),
        scene_ecs_query_structure: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query_structure.rs",
        ),
        scene_ecs_systems: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs",
        ),
        shader_prewarm_manifest: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs",
        ),
        status_output_expected_slices: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs",
        ),
        status_output_row_data: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs",
        ),
        status_output_row_data_runtime_15: [
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            ),
            read_runtime_src(
                "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            ),
        ]
        .join("\n"),
        ui_shared_core: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core.rs",
        ),
        ui_v2_asset: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/ui_v2_asset.rs",
        ),
    }
}
