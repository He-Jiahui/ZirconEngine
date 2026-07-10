#[test]
fn runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass() {
    let runtime_13_plan = runtime_plan_source_with_archive(
        "13",
        include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    ),
    );
    let runtime_13_plan = runtime_13_plan.as_str();
    let runtime_index = runtime_index_with_numbered_archives(include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/index.md"
    ));
    let runtime_index = runtime_index.as_str();
    let function_ledger =
        include_str!("../../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_13_plan),
        Some("in_progress"),
        "Runtime 13 should stay in progress until script binding validation closes"
    );

    for required_row in [
        "1.1 清册守卫",
        "1.2 capability 测试",
        "2.1 句柄失效语义",
        "2.2 访问路径收束",
    ] {
        let row_anchor = format!("| {required_row} |");
        let row = runtime_13_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 13 should keep status row `{required_row}`"));
        assert_contains_all(
            "Runtime 13 pending status row",
            row,
            &["code_static_pending_cargo", "Cargo"],
        );
    }

    assert_contains_all(
        "Runtime 13 validation gate commands",
        runtime_13_plan,
        &[
            "cargo test -p zircon_runtime --lib script --locked -- --nocapture",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "host_function_registry_matches_documented_ledger",
            "host_capability_representatives_are_declared_on_registered_modules",
            "script_held_entity_handle_reports_invalid_after_despawn",
            "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
        ],
    );

    let runtime_13_index_row =
        runtime_index_row_for(runtime_index, "13-script-binding-and-reflection.md");
    assert_contains_all(
        "Runtime 13 index row",
        runtime_13_index_row,
        &[
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "script filters",
            "Cargo 待 active lane 清空",
        ],
    );

    let runtime_13_problem_row =
        runtime_index_problem_row_for(runtime_index, "P16", "script binding");
    assert_contains_all(
        "Runtime index P16 row",
        runtime_13_problem_row,
        &[
            "function_ledger.md",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "Cargo 待 active lane 清空",
        ],
    );

    assert_contains_all(
        "Runtime script host ledger doc",
        function_ledger,
        &[
            "6 host modules, 52 fixed host functions, and 2 fixed script type descriptors",
            "host_function_registry_matches_documented_ledger",
            "host_capability_representatives_are_declared_on_registered_modules",
            "script_held_entity_handle_reports_invalid_after_despawn",
            "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
            "pending: cargo test -p zircon_runtime --lib script --locked -- --nocapture",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 13 gate",
        review,
        &[
            "Runtime 13 Script Binding Guard",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "script filters",
        ],
    );
}
