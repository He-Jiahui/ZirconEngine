#[test]
fn runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation() {
    let runtime_07_plan = runtime_plan_source_with_archive(
        "07",
        include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    ),
    );
    let runtime_07_plan = runtime_07_plan.as_str();
    let runtime_index = runtime_index_with_numbered_archives(include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/index.md"
    ));
    let runtime_index = runtime_index.as_str();
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
        Some("in_progress"),
        "Runtime 07 should stay in progress until performance Cargo/profiling/FPS validation closes"
    );

    for (row_name, status_anchor) in [
        ("0.3 帧分解 span", "frame_spans_static_passed_trace_pending"),
        (
            "1.1 计数点",
            "scoped_counter_points_extract_implemented_cargo_blocked",
        ),
        (
            "1.2 计数断言",
            "named_assertions_static_passed_cargo_blocked",
        ),
        (
            "1.3 热点清单",
            "inventory_scaffold_static_passed_pending_authoritative_values",
        ),
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_07_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 07 should keep status row `{row_name}`"));
        assert_contains_all(
            "Runtime 07 pending status row",
            row,
            &[status_anchor, "Cargo"],
        );
        assert!(
            !row.contains("completed |"),
            "Runtime 07 row `{row_name}` must not claim completed before performance validation closes"
        );
    }

    assert_contains_all(
        "Runtime 07 validation gate commands",
        runtime_07_plan,
        &[
            "cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features backend-zr-vm --locked -- --nocapture --test-threads=1",
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib extract --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib ecs_query --locked -- --nocapture",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
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
        &[
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "extract/ecs_query/performance profiling/FPS gates",
            "Cargo/profiling/FPS 待",
        ],
    );

    let runtime_07_problem_row =
        runtime_index_problem_row_for(runtime_index, "P5", "runtime performance");
    assert_contains_all(
        "Runtime index P5 row",
        runtime_07_problem_row,
        &[
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "profiling 构建",
            "runtime 真实后端验证",
        ],
    );

    assert_contains_all(
        "Runtime 07 hotspot inventory doc",
        hotspot_doc,
        &[
            "Evidence Gate",
            "No Runtime 07 M2 optimization slice may start from an unmeasured suspicion",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "extract/ecs_query/performance profiling/FPS gates",
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
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "extract/ecs_query/performance profiling/FPS gates",
        ],
    );
}
