#[test]
fn runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass() {
    let runtime_14_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");

    assert_eq!(
        frontmatter_status(runtime_14_plan),
        Some("in_progress"),
        "Runtime 14 should stay in progress until module-family Cargo/rustc gates close"
    );

    for required_m1_row in [
        "navigation 文件集守卫",
        "engine_module declared-layer 守卫",
        "diagnostic_log 单桥接守卫",
        "animation backlog/非目标守卫",
        "crate 根四族席位总守卫",
    ] {
        let row_anchor = format!("| {required_m1_row} |");
        let row = runtime_14_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 14 should keep status row `{required_m1_row}`"));
        assert_contains_all(
            "Runtime 14 M1 status row",
            row,
            &["code_static_pending_cargo", "Cargo"],
        );
    }

    let runtime_14_index_row =
        runtime_index_row_for(runtime_index, "14-runtime-module-family-closeout.md");
    assert_contains_all(
        "Runtime 14 index row",
        runtime_14_index_row,
        &[
            "Runtime 14 animation/navigation/diagnostic_log/engine_module 四族裁决",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
            "完整 Runtime 14 filters/full sweep 仍 pending",
        ],
    );

    let runtime_14_problem_row =
        runtime_index_problem_row_for(runtime_index, "P17", "module-family");
    assert_contains_all(
        "Runtime index P17 row",
        runtime_14_problem_row,
        &[
            "runtime_14_module_family_root_seats_match_documented_judgements",
            "Cargo/rustc",
            "待 active lane 清空",
        ],
    );
}
