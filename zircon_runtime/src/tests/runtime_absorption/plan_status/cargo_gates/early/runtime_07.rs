#[test]
fn runtime_07_performance_hotpath_records_completed_authoritative_validation() {
    let runtime_07_plan = runtime_plan_source_with_archive(
        "07",
        include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    ),
    );
    let runtime_07_plan = runtime_07_plan.as_str();
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let ecs_doc = include_str!("../../../../../../../docs/zircon_runtime/scene/ecs.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_07_plan),
        Some("completed"),
        "Runtime 07 should be complete after authoritative FPS, trace and ECS/extract evidence closes"
    );

    assert_contains_all(
        "Runtime 07 completion evidence",
        runtime_07_plan,
        &[
            "frame_spans_trace_accepted_completed",
            "scoped_counter_points_runtime_published_completed",
            "named_assertions_behavior_accepted_completed",
            "authoritative_inventory_completed",
            "9.521868%",
            "headless_session_tick_publishes_ecs_frame_diagnostics",
            "frame_extract_rebuild` 2/2",
            "`ecs_query` 58/58",
        ],
    );

    assert_contains_all(
        "Runtime 07 validation gate commands",
        runtime_07_plan,
        &[
            "cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features backend-zr-vm --locked -- --nocapture --test-threads=1",
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib extract --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib ecs_query --locked -- --nocapture",
            "runtime_07_performance_hotpath_records_completed_authoritative_validation",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
            "query_state_reuses_archetype_matches_across_unchanged_frames",
            "change_detection_scan_skips_unmarked_archetypes",
            "frame_extract_rebuild_skips_unchanged_entities",
            "runtime_frame_schedule_stage.<SystemStage>",
        ],
    );

    let runtime_07_index_row =
        runtime_index_row_for(runtime_index, "07-runtime-performance-hotpath.md");
    assert_contains_all(
        "Runtime 07 index row",
        runtime_07_index_row,
        &["completed", "双次 Vampire FPS", "ECS/extract 计数"],
    );

    let runtime_07_problem_row =
        runtime_index_problem_row_for(runtime_index, "P5", "runtime performance");
    assert_contains_all(
        "Runtime index P5 row",
        runtime_07_problem_row,
        &[
            "性能热路径、权威 FPS、trace 与 ECS/extract 诊断已完成",
            "共享工作区全包编译",
        ],
    );

    assert_contains_all(
        "Runtime 07 hotspot inventory doc",
        hotspot_doc,
        &[
            "Evidence Gate",
            "No Runtime 07 M2 optimization slice may start from an unmeasured suspicion",
            "9.521868%",
            "EcsFramePerformanceDiagnostics::publish(...)",
        ],
    );
    assert_contains_all(
        "Runtime dynamic session frame diagnostics doc",
        dynamic_session_doc,
        &[
            "runtime_frame_time_update",
            "runtime_frame_extract",
            "runtime_frame_submit",
            "runtime_frame_schedule_stage.<SystemStage>",
            "frame_extract_rebuild_skips_unchanged_entities",
        ],
    );
    assert_contains_all(
        "Runtime ECS profiling doc",
        ecs_doc,
        &[
            "SceneScheduleRunner::run_stage(...)",
            "runtime_frame_schedule_stage.<SystemStage>",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 07 gate",
        review,
        &[
            "Runtime 07 Performance Hotpath Guard",
            "runtime_07_performance_hotpath_records_completed_authoritative_validation",
            "9.521868%",
        ],
    );
}
