#[test]
fn runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass() {
    let runtime_11_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let job_system_doc =
        include_str!("../../../../../../../docs/zircon_runtime/core/job_system.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let runtime_05_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );

    assert_eq!(
        frontmatter_status(runtime_11_plan),
        Some("in_progress"),
        "Runtime 11 should stay in progress until tasks/ecs_schedule/worker_pool/rayon validation closes"
    );

    for (row_name, required_anchors) in [
        (
            "1.1 句柄与依赖",
            &[
                "code_static_pending_cargo",
                "tasks",
                "cargo test -p zircon_runtime --lib tasks --locked -- --nocapture",
            ][..],
        ),
        (
            "1.2 parallel_for",
            &[
                "code_static_pending_cargo",
                "tasks",
                "cargo test -p zircon_runtime --lib tasks --locked -- --nocapture",
            ][..],
        ),
        (
            "2.1 剔除旁路收编",
            &[
                "pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner",
                "render-owner-pending-runtime-11-m2-1-cutover",
                "actual graphics cutover not executed",
            ][..],
        ),
        (
            "2.2 rayon 守卫",
            &[
                "code_static_pending_render_cutover_cargo",
                "M2.2 Cargo 仍待",
                "render owner",
            ][..],
        ),
        (
            "2.3 ECS 批次依赖化",
            &["code_static_pending_cargo", "ecs_schedule", "Cargo"][..],
        ),
        (
            "2.4 asset 线程裁决",
            &["code_static_pending_cargo", "worker_pool", "Cargo"][..],
        ),
        (
            "3.1 调度诊断",
            &["code_static_pending_cargo", "tasks", "Cargo"][..],
        ),
        (
            "3.2 压测锚",
            &[
                "code_static_pending_cargo",
                "tasks",
                "cargo test -p zircon_runtime --lib tasks --locked -- --nocapture",
            ][..],
        ),
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_11_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 11 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 11 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 11 validation gate commands",
        runtime_11_plan,
        &[
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib tasks --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib job --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib rayon --locked",
            "cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib worker_pool --locked",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
        ],
    );

    let runtime_11_index_row = runtime_index_row_for(runtime_index, "11-job-system-task-model.md");
    assert_contains_all(
        "Runtime 11 index row",
        runtime_11_index_row,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "tasks/ecs_schedule/worker_pool/rayon",
            "Cargo",
        ],
    );

    let runtime_11_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P14 |"))
        .expect("Runtime index should keep the P14 JobSystem problem row");
    assert_contains_all(
        "Runtime index P14 row",
        runtime_11_problem_row,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "parallel_frustum",
            "tasks/ecs_schedule/worker_pool/rayon",
        ],
    );

    assert_contains_all(
        "Runtime JobSystem module doc",
        job_system_doc,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "tasks/ecs_schedule/worker_pool/rayon",
            "parallel_frustum.rs",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "Runtime 11 `tasks/ecs_schedule/worker_pool/rayon` Cargo gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 11 gate",
        review,
        &[
            "Runtime 11 JobSystem Cargo Gate",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "tasks/ecs_schedule/worker_pool/rayon",
        ],
    );
}
