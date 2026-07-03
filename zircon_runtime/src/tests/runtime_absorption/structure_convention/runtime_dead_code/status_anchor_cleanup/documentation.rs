use super::*;

#[test]
fn runtime_15_runtime_dead_code_documentation_anchors_use_folder_owner() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                CURRENT_ROOT_OWNER,
                CURRENT_RUNTIME_UI_OWNER,
                CURRENT_PRODUCTION_SCAN_OWNER,
            ],
        );
        assert!(
            !source.contains(STALE_FLAT_OWNER),
            "{label} should not point current status/docs at the deleted flat runtime dead-code owner"
        );
    }

    assert_contains_all(
        "Runtime 15 plan owner inventory",
        &runtime_15_plan,
        &[
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs",
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
        ],
    );
    assert_contains_all(
        "module convention owner inventory",
        &module_doc,
        &[
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs",
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
        ],
    );
}
