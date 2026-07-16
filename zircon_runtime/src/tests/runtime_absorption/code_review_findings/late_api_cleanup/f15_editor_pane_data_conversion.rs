#[test]
fn review_f15_editor_pane_data_conversion_top_row_uses_projection_owners() {
    let pane_mod = include_str!(
        "../../../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs"
    );
    let template_node_projection = include_str!(
        "../../../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs"
    );
    let animation_projection = include_str!(
        "../../../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs"
    );
    let apply_pane_conversion = include_str!(
        "../../../../../../zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs"
    );
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_15 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let editor_workbench_doc =
        include_str!("../../../../../../docs/editor-and-tooling/editor-workbench-shell.md");
    let f15_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F15 |"))
        .expect("engine-code-review findings should keep a top-level F15 row");
    assert!(
        f15_row.contains("pane_data_conversion")
            && f15_row.ends_with("| Runtime 15 + Editor UI 10 |"),
        "F15 overview row should keep only the finding and delegated owners"
    );

    for required in [
        "editor `pane_data_conversion` 投影函数样板复制已由 child projection owners 收束",
        "pane_data_conversion/mod.rs",
        "template_node_projection.rs",
        "animation_projection.rs",
        "apply_presentation/pane_conversion.rs",
        "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners",
    ] {
        assert!(
            review_findings.contains(required),
            "F15 numbered review evidence should record current projection-owner state `{required}`"
        );
    }
    for stale_finding in [
        "editor/.../pane_data_conversion/mod.rs:74",
        "to_host_contract_pane` ~228 行",
        "animation_template_projection` ~326 行",
        "抽 `project_nodes<T>()` 泛型 helper + 按 header/body/fields 拆",
    ] {
        assert!(
            !f15_row.contains(stale_finding),
            "F15 top review row should not keep stale unresolved finding text `{stale_finding}`"
        );
    }

    for required in [
        "mod animation_projection;",
        "mod template_node_projection;",
        "pub(crate) use self::animation_projection::",
        "use self::template_runtime_projection::",
    ] {
        assert!(
            pane_mod.contains(required),
            "pane_data_conversion root should retain child projection-owner wiring `{required}`"
        );
    }
    for moved_owner in [
        "fn animation_template_projection(",
        "fn to_host_contract_animation_editor_pane(",
        "fn to_host_contract_pane(",
        "pub(super) fn project_nodes<",
    ] {
        assert!(
            !pane_mod.contains(moved_owner),
            "pane_data_conversion root should not regain moved owner `{moved_owner}`"
        );
    }
    for required in [
        "pub(super) fn project_nodes<T, F>(",
        "pub(super) fn project_node_vec<T, F>(",
    ] {
        assert!(
            template_node_projection.contains(required),
            "template node projection should own shared node helper `{required}`"
        );
    }
    assert!(
        animation_projection.contains("fn animation_template_projection(")
            && animation_projection.contains("PanePayload::AnimationSequenceV1")
            && animation_projection.contains("PanePayload::AnimationGraphV1"),
        "animation projection owner should retain sequence and graph payload conversion"
    );
    assert!(
        apply_pane_conversion.contains("pub(super) fn to_host_contract_pane(")
            && apply_pane_conversion.contains("has_animation_payload")
            && apply_pane_conversion.contains("to_host_contract_animation_editor_pane("),
        "apply presentation owner should retain pane routing"
    );

    for doc_anchor in [
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard",
        "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
        "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_15.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || editor_workbench_doc.contains(doc_anchor),
            "F15 docs should record `{doc_anchor}`"
        );
    }
}
