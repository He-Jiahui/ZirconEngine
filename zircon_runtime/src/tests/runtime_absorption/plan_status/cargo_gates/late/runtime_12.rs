#[test]
fn runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation() {
    let runtime_12_plan = runtime_plan_source_with_archive("12", include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    ));
    let runtime_12_plan = runtime_12_plan.as_str();
    let runtime_index = runtime_index_with_numbered_archives(include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/index.md"
    ));
    let runtime_index = runtime_index.as_str();
    let input_doc = include_str!("../../../../../../../docs/zircon_runtime/input/input_state.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_12_plan),
        Some("in_progress"),
        "Runtime 12 should stay in progress until input/action/gamepad validation closes"
    );

    for (row_name, status_anchor) in [
        (
            "0.1 链路与帧语义",
            "input_frame_contract_static_passed_cargo_pending",
        ),
        (
            "1.1 动作映射设计",
            "action_contract_static_passed_cargo_pending",
        ),
        (
            "1.2 最小实现",
            "action_evaluator_static_passed_cargo_pending",
        ),
        (
            "2.1 gamepad 桥接",
            "gamepad_bridge_static_passed_cargo_pending",
        ),
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_12_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 12 should keep status row `{row_name}`"));
        assert_contains_all(
            "Runtime 12 pending status row",
            row,
            &[status_anchor, "Cargo"],
        );
    }

    assert_contains_all(
        "Runtime 12 validation gate commands",
        runtime_12_plan,
        &[
            "cargo test -p zircon_runtime --lib input --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib action_map --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib gamepad --locked -- --nocapture",
            "cargo test -p zircon_app --locked",
            "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
        ],
    );

    let runtime_12_index_row =
        runtime_index_row_for(runtime_index, "12-input-stack-and-action-mapping.md");
    assert_contains_all(
        "Runtime 12 index row",
        runtime_12_index_row,
        &[
            "Runtime 12 输入契约/runtime/tests",
            "input/action_map/gamepad/app filters",
            "Cargo 待 active lane 清空",
        ],
    );

    assert_contains_all(
        "Runtime input module doc",
        input_doc,
        &[
            "Frame Input Contract",
            "DefaultInputManager::begin_frame()",
            "InputActionEvaluator",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 12 gate",
        review,
        &[
            "Runtime 12 Input Stack Guard",
            "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
            "input/action_map/gamepad/app",
        ],
    );
}
