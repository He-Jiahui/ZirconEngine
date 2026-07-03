use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_production_file_budget_guard_child_owner_split() {
    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/production_file_budget.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    );

    assert_contains_all(
        "production-file-budget parent mounts all child owners",
        &parent,
        &[
            "mod module_layout;",
            "mod rhi_wgpu_command_validation;",
            "mod rhi_wgpu_ui_surface_render_setup;",
            "mod render_scene_world;",
            "mod render_shadow;",
            "mod render_stats_graph;",
            "mod render_stats_product_tests;",
            "mod native_host_api_adapter;",
            "mod scene_fixed_lights;",
            "mod ui_text_layout;",
            "fn read_runtime_src(",
            "fn read_repo(",
        ],
    );
    for moved_guard in [
        "fn runtime_15_ui_text_layout_engine_visual_order_is_child_owner",
        "fn runtime_15_rhi_wgpu_command_validation_state_is_child_owner",
        "fn runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners",
        "fn runtime_15_render_stats_graph_execution_resources_are_child_owner",
        "fn runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner",
        "fn runtime_15_scene_world_render_visibility_input_is_child_owner",
        "fn runtime_15_shadow_plan_view_projection_is_child_owner",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "production_file_budget.rs should mount child owner instead of defining {moved_guard}"
        );
    }

    for relative in [
        "tests/runtime_absorption/structure_convention/production_file_budget.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/module_layout.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_command_validation.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_ui_surface_render_setup.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/render_scene_world.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/render_shadow.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/render_stats_graph.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/render_stats_product_tests.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/native_host_api_adapter.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/scene_fixed_lights.rs",
        "tests/runtime_absorption/structure_convention/production_file_budget/ui_text_layout.rs",
    ] {
        let source = read_runtime_src(relative);
        let line_count = source.lines().count();
        assert!(
            line_count < 300,
            "{relative} should stay below the Runtime 15 guard-owner budget after the child split; got {line_count} lines"
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
                "Runtime 15 M3 production file budget guard child-owner split",
                "runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/production_file_budget.rs",
                "structure_convention/production_file_budget/module_layout.rs",
                "runtime_15_production_file_budget_guard_child_owner_split",
            ],
        );
    }
}
