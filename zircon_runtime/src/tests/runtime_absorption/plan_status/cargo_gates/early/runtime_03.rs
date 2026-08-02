#[test]
fn runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation() {
    let runtime_03_plan = runtime_plan_source_with_archive("03", include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
    ));
    let runtime_03_plan = runtime_03_plan.as_str();
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
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
        Some("completed"),
        "Runtime 03 should be complete after current Runtime and zircon_app validation closes"
    );

    assert_contains_all(
        "Runtime 03 completion evidence",
        runtime_03_plan,
        &[
            "runtime_03_all_declared_cargo_gates_passed_completed",
            "`ecs_schedule` 77/77",
            "`tests::time::` 4/4",
            "`session` 165 passed / 0 failed / 10 ignored",
            "`schedule_parallel` 15/15",
            "主测试 135 passed / 0 failed / 1 ignored",
            "PBR viewer 15/15",
        ],
    );

    assert_contains_all(
        "Runtime 03 validation gate commands",
        runtime_03_plan,
        &[
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter ecs_schedule",
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter session",
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_app",
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter fixed_update",
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter tests::time::",
            r".\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter schedule_parallel",
            "runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation",
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
            "completed",
            "Runtime filters 77/77、4/4、165/0/10 ignored、15/15",
            "`zircon_app` 135/0/1 ignored + PBR viewer 15/15",
        ],
    );

    let runtime_03_problem_row =
        runtime_index_problem_row_for(runtime_index, "P3", "schedule/frame-loop");
    assert_contains_all(
        "Runtime index P3 row",
        runtime_03_problem_row,
        &[
            "schedule/time/frame-loop 单一权威已收口",
            "当前 Runtime 四组过滤门",
            "`zircon_app` 全包门槛已闭合",
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
            "Runtime 03 Schedule Frame-Loop Completion Guard",
            "runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation",
            "Runtime 03 as `completed`",
            "135 passed / 0 failed / 1 ignored",
        ],
    );
}
