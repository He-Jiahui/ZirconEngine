#[test]
fn runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation() {
    let runtime_09_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let ui_architecture_doc =
        include_str!("../../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_09_plan),
        Some("in_progress"),
        "Runtime 09 should stay in progress until UI owner and Cargo validation closes"
    );

    for (row_name, required_anchors) in [
        (
            "0.1 模块边界图",
            &[
                "completed_static_passed",
                "runtime_09_m0_ui_architecture_static_passed",
                "Cargo 未启动",
            ][..],
        ),
        (
            "0.2 v2 裁决",
            &[
                "completed_static_passed",
                "v2-replacement-mainline",
                "runtime_09_v2_verdict_matches_runtime_and_interface_modules",
            ][..],
        ),
        ("1.1 路由单点", &["待开始"][..]),
        ("1.2 legacy 处置", &["待开始"][..]),
        ("2.1 taffy 单入口", &["待开始"][..]),
        ("2.2 虚拟化边界", &["待开始"][..]),
        ("3.1 模板边界", &["待开始"][..]),
    ] {
        let row = runtime_09_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 09 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 09 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 09 validation gate commands",
        runtime_09_plan,
        &[
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib ui --locked",
            "cargo test -p zircon_runtime --lib input --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib naming_boundary --locked",
            "cargo test -p zircon_runtime --lib layout --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib template --locked -- --nocapture",
            "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
        ],
    );

    let runtime_09_index_row =
        runtime_index_row_for(runtime_index, "09-ui-subsystem-architecture.md");
    assert_contains_all(
        "Runtime 09 index row",
        runtime_09_index_row,
        &[
            "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
            "ui/input/naming_boundary/layout/template",
            "editor UI owner/Cargo lanes",
        ],
    );

    let runtime_09_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P12 |"))
        .expect("Runtime index should keep the P12 UI subsystem problem row");
    assert_contains_all(
        "Runtime index P12 row",
        runtime_09_problem_row,
        &[
            "runtime_absorption::ui_architecture",
            "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
            "editor UI owner",
        ],
    );

    assert_contains_all(
        "Runtime UI architecture doc",
        ui_architecture_doc,
        &[
            "runtime_09_m0_ui_architecture_static_passed",
            "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
            "ui/input/naming_boundary/layout/template",
            "Cargo has not been run",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
            "Runtime 09 `ui/input/naming_boundary/layout/template` owner/Cargo gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 09 gate",
        review,
        &[
            "Runtime 09 UI Cargo Gate",
            "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
            "ui/input/naming_boundary/layout/template",
        ],
    );
}
