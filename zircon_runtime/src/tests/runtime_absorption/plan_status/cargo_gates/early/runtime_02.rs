#[test]
fn runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation() {
    let runtime_02_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let root_surface_doc =
        include_str!("../../../../../../../docs/zircon_runtime/core/root_surface.md");
    let generated_boundary_doc =
        include_str!("../../../../../../../docs/engine-architecture/generated-code-boundary.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let runtime_05_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );

    assert_eq!(
        frontmatter_status(runtime_02_plan),
        Some("in_progress"),
        "Runtime 02 should stay in progress until core/root/generated validation closes"
    );

    for (row_name, required_anchors) in [
        ("M2 | 测试阶段", &["进行中", "Cargo", "render/graphics"][..]),
        (
            "M3 | 3.3 root graphics alias block removal",
            &[
                "graphics_alias_block_removed_static_passed_cargo_pending",
                "crate_visible_graphics_reexport_count = 0",
                "crate-visible graphics alias debt 0/0",
            ][..],
        ),
        (
            "M4 | 4.2 行为迁回与守卫",
            &[
                "代码完成，Cargo 测试待重跑",
                "generated",
                "export_build_plan",
            ][..],
        ),
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_02_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 02 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 02 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 02 validation gate commands",
        runtime_02_plan,
        &[
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib core:: --locked",
            "cargo test -p zircon_runtime --lib runtime_absorption --locked -- --nocapture",
            "cargo test -p zircon_app --locked",
            "cargo check -p zircon_editor --lib --locked",
            "cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked",
            "cargo test -p zircon_runtime --lib generated --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib export_build_plan --locked",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
        ],
    );

    let runtime_02_index_row =
        runtime_index_row_for(runtime_index, "02-core-spine-and-root-surface.md");
    assert_contains_all(
        "Runtime 02 index row",
        runtime_02_index_row,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "M2/M4 全量 Cargo 回归",
            "root alias cutover 已静态完成但包级 Cargo 待验证",
        ],
    );

    let runtime_02_problem_row =
        runtime_index_problem_row_for(runtime_index, "P2", "core spine/root surface");
    assert_contains_all(
        "Runtime index P2 row",
        runtime_02_problem_row,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "graphics_alias_block_removed_static_passed_cargo_pending",
            "crate-visible graphics alias debt 0/0",
            "Cargo default/lib-test",
        ],
    );

    let runtime_02_generated_problem_row =
        runtime_index_problem_row_for(runtime_index, "P8", "generated-code");
    assert_contains_all(
        "Runtime index P8 row",
        runtime_02_generated_problem_row,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "m1_gate_status=classified-and-clear",
            "Cargo 测试阶段仍受",
        ],
    );

    assert_contains_all(
        "Runtime root surface doc",
        root_surface_doc,
        &[
            "graphics_alias_block_removed_static_passed_cargo_pending",
            "crate-visible graphics alias debt 0/0",
            "core/root/generated/export_build_plan/app/editor/plugin",
        ],
    );

    assert_contains_all(
        "Runtime generated boundary doc",
        generated_boundary_doc,
        &[
            "classified-and-clear",
            "generated_code_boundary.m1_gate_status",
            "migration_debt_location_count = 0",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "Runtime 02 `core/root/generated/export_build_plan/app/editor/plugin` gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 02 gate",
        review,
        &[
            "Runtime 02 Core/Root/Generated Gate",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "core/root/generated/export_build_plan/app/editor/plugin",
        ],
    );
}
