use super::*;

#[test]
fn runtime_15_module_layout_historical_status_rows_are_current() {
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let status_map = read_runtime_src(ROOT_RUNTIME_STATUS_MAP_PATH);
    let date_map = read_runtime_src(ROOT_RUNTIME_DATE_MAP_PATH);

    let historical_anchors = [
        HISTORICAL_STATUS_NAME,
        HISTORICAL_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data.rs",
        "structure_convention/test_file_budget/row_data/module_layout.rs",
        HISTORICAL_GUARD_NAME,
    ];
    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &historical_anchors);
    }
    assert_contains_all(
        "status-output Runtime 15 M3 production support row data records historical module-layout split",
        &production_guard_support,
        &historical_anchors,
    );
    assert_contains_all(
        "Runtime 15 expected status map owns historical module-layout split",
        &status_map,
        &[HISTORICAL_STATUS_NAME, HISTORICAL_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 expected date map owns historical module-layout split",
        &date_map,
        &[HISTORICAL_STATUS_NAME, "2026-06-24"],
    );
}
