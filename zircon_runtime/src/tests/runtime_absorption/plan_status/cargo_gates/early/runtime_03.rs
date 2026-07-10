#[test]
fn runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation() {
    let runtime_03_plan = runtime_plan_source_with_archive("03", include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
    ));
    let runtime_03_plan = runtime_03_plan.as_str();
    let runtime_index = runtime_index_with_numbered_archives(include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/index.md"
    ));
    let runtime_index = runtime_index.as_str();
    let frame_schedule_doc =
        include_str!("../../../../../../../docs/zircon_runtime/core/frame_schedule.md");
    let schedule_parallel_doc = include_str!(
        "../../../../../../../docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md"
    );
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_03_plan),
        Some("in_progress"),
        "Runtime 03 should stay in progress until schedule/frame-loop Cargo validation closes"
    );

    for row_name in [
        "1.1 隐式顺序显式化",
        "1.2 UI extract 合法旁路契约",
        "2.1 单次 `RuntimeTimeAdvance` 接通",
        "2.2 插值因子",
        "3.1 开关与计数",
        "3.2 一致性与收益",
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_03_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 03 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 03 pending status row", row, &["Cargo", "待"]);
    }

    assert_contains_all(
        "Runtime 03 validation gate commands",
        runtime_03_plan,
        &[
            "cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib session --locked",
            "cargo test -p zircon_app --locked",
            "cargo test -p zircon_runtime --lib fixed_update --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib tests::time:: --locked",
            "cargo test -p zircon_runtime --lib schedule_parallel --locked -- --nocapture",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration",
            "session_ui_extract_remains_documented_dynamic_session_side_path",
            "world_driver_consumes_runtime_time_advance_without_advancing_clocks_again",
            "fixed_step_plan_reports_overstep_fraction_in_unit_range",
            "schedule_parallel_executor_can_run_parallel_batches_serially_with_report",
            "schedule_parallel_execution_report_records_diagnostic_counts",
            "representative_schedule_produces_multi_system_parallel_batches",
            "parallel_and_serial_execution_reach_identical_world_state",
        ],
    );

    let runtime_03_index_row =
        runtime_index_row_for(runtime_index, "03-schedule-and-frame-loop-alignment.md");
    assert_contains_all(
        "Runtime 03 index row",
        runtime_03_index_row,
        &[
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "ecs_schedule/time/session/schedule_parallel Cargo gates",
            "Cargo 待 active lanes 清空",
        ],
    );

    let runtime_03_problem_row =
        runtime_index_problem_row_for(runtime_index, "P3", "schedule/frame-loop");
    assert_contains_all(
        "Runtime index P3 row",
        runtime_03_problem_row,
        &[
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "ecs_schedule/time/session/schedule_parallel",
            "Cargo 回归待运行",
        ],
    );

    assert_contains_all(
        "Runtime frame schedule doc",
        frame_schedule_doc,
        &[
            "Runtime Frame Schedule",
            "session_ui_extract_remains_documented_dynamic_session_side_path",
            "WorldDriver",
            "RuntimeTimeAdvance",
            "fixed_step_plan_reports_overstep_fraction_in_unit_range",
            "schedule.parallel_batches",
        ],
    );
    assert_contains_all(
        "Runtime schedule parallel executor doc",
        schedule_parallel_doc,
        &[
            "ScheduleParallelExecutionReport",
            "schedule_parallel_execution_report_records_diagnostic_counts",
            "parallel_and_serial_execution_reach_identical_world_state",
            "schedule_parallel_batches_chain_through_job_handles",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 03 gate",
        review,
        &[
            "Runtime 03 Schedule Frame-Loop Guard",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "ecs_schedule/time/session/schedule_parallel",
        ],
    );
}
