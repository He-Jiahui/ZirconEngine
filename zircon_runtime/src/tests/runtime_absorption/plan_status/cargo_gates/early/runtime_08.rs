#[test]
fn runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation() {
    let runtime_08_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let ecs_doc = include_str!("../../../../../../../docs/zircon_runtime/scene/ecs.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_08_plan),
        Some("in_progress"),
        "Runtime 08 should stay in progress until ECS data-kernel validation closes"
    );

    for row_name in [
        "1.1 生命周期测试矩阵",
        "2.1 观察者时序",
        "2.2 命令队列错误路径",
        "3.1 双通道定稿",
        "3.2 tick 回绕",
    ] {
        let row = runtime_08_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 08 should keep status row `{row_name}`"));
        assert_contains_all(
            "Runtime 08 pending status row",
            row,
            &["code_complete_pending_cargo", "Cargo"],
        );
    }

    assert_contains_all(
        "Runtime 08 validation gate commands",
        runtime_08_plan,
        &[
            "cargo test -p zircon_runtime --lib entity --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib observer --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib command --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib change_tick --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib messages --locked",
            "cargo test -p zircon_runtime --lib ecs --locked",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
        ],
    );

    let runtime_08_index_row =
        runtime_index_row_for(runtime_index, "08-ecs-kernel-data-alignment.md");
    assert_contains_all(
        "Runtime 08 index row",
        runtime_08_index_row,
        &[
            "Runtime 08 ECS 数据面守卫",
            "entity/observer/command/messages/change_tick/ecs filters",
            "Cargo 待活动 lanes 清空后运行",
        ],
    );

    let runtime_08_problem_row =
        runtime_index_problem_row_for(runtime_index, "P11", "ECS data-kernel");
    assert_contains_all(
        "Runtime index P11 row",
        runtime_08_problem_row,
        &[
            "测试已落地待 Cargo",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
        ],
    );

    assert_contains_all(
        "Runtime ECS module doc",
        ecs_doc,
        &[
            "Runtime 08 Data-Kernel Alignment Verdict",
            "despawned_entity_handle_is_rejected_by_world_access",
            "lifecycle_observer_fires_immediately_during_component_mutation",
            "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
            "events_require_explicit_update_and_keep_next_queue_hidden",
            "change_tick_comparison_survives_wraparound",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 08 gate",
        review,
        &[
            "Runtime 08 ECS Data-Kernel Guard",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "entity/observer/command/messages/change_tick/ecs",
        ],
    );
}
