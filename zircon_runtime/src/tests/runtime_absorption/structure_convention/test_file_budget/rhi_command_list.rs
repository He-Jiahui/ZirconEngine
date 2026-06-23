use super::*;

fn runtime_15_rhi_command_list_tests_are_folder_backed() {
    let parent = read_runtime_src("rhi/tests/command_list.rs");
    let basic_commands = read_runtime_src("rhi/tests/command_list/basic_commands.rs");
    let bind_groups = read_runtime_src("rhi/tests/command_list/bind_groups.rs");
    let raster_draws = read_runtime_src("rhi/tests/command_list/raster_draws.rs");
    let vertex_index_state = read_runtime_src("rhi/tests/command_list/vertex_index_state.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "RHI command list parent test module mounts",
        &parent,
        &[
            "mod basic_commands;",
            "mod bind_groups;",
            "mod raster_draws;",
            "mod vertex_index_state;",
            "fn create_compute_pipeline",
            "fn create_raster_pipeline_with_layout_and_vertex_input",
            "fn begin_default_render_pass",
        ],
    );

    for moved_guard in [
        "fn command_list_records_compute_dispatch_and_submit_validates_pipeline",
        "fn command_list_submit_validates_compute_pipeline_bind_groups",
        "fn command_list_records_raster_draws_and_submit_validates_bound_buffers",
        "fn command_list_raster_draw_submit_validates_vertex_and_index_buffer_state",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "rhi/tests/command_list.rs should mount child test owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "RHI command list basic child owns queue/copy/compute contracts",
        &basic_commands,
        &[
            "fn command_list_keeps_queue_class_and_label",
            "fn command_list_records_buffer_copy_commands_and_submit_validates_resources",
            "fn command_list_records_compute_dispatch_and_submit_validates_pipeline",
        ],
    );
    assert_contains_all(
        "RHI command list bind group child owns pipeline-layout contracts",
        &bind_groups,
        &[
            "fn command_list_records_bind_groups_and_submit_validates_raster_pipeline_layout",
            "fn command_list_submit_validates_compute_pipeline_bind_groups",
            "fn command_list_submit_validates_bind_group_layout_compatibility",
        ],
    );
    assert_contains_all(
        "RHI command list raster child owns draw validation contracts",
        &raster_draws,
        &[
            "fn command_list_records_raster_draws_and_submit_validates_bound_buffers",
            "fn command_list_allows_generated_vertex_draws_without_vertex_buffers",
            "fn command_list_raster_draw_submit_validates_pipeline_queue_and_counts",
        ],
    );
    assert_contains_all(
        "RHI command list vertex/index child owns buffer-state contracts",
        &vertex_index_state,
        &[
            "fn command_list_raster_draw_submit_validates_vertex_and_index_buffer_state",
            "fn command_list_buffer_copy_submit_validates_usage_flags",
        ],
    );

    for (path, source) in [
        ("rhi/tests/command_list.rs", parent.as_str()),
        (
            "rhi/tests/command_list/basic_commands.rs",
            basic_commands.as_str(),
        ),
        (
            "rhi/tests/command_list/bind_groups.rs",
            bind_groups.as_str(),
        ),
        (
            "rhi/tests/command_list/raster_draws.rs",
            raster_draws.as_str(),
        ),
        (
            "rhi/tests/command_list/vertex_index_state.rs",
            vertex_index_state.as_str(),
        ),
    ] {
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
                "Runtime 15 M3 RHI command list test folder split",
                "runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked",
                "rhi/tests/command_list/basic_commands.rs",
                "rhi/tests/command_list/vertex_index_state.rs",
                "runtime_15_rhi_command_list_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 RHI command list test folder split",
            "runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked",
            "rhi/tests/command_list.rs",
            "rhi/tests/command_list/basic_commands.rs",
            "runtime_15_rhi_command_list_tests_are_folder_backed",
        ],
    );
}
