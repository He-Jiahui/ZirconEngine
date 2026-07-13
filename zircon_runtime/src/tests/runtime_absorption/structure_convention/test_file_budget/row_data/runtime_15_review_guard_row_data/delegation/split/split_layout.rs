use super::*;

#[test]
fn runtime_15_review_guard_row_data_delegation_split_layout_guard_is_folder_backed() {
    let parent = read_runtime_src(DELEGATION_SPLIT_LAYOUT_CHILD_PATH);
    let route_mounts = read_runtime_src(DELEGATION_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH);
    let status_current = read_runtime_src(DELEGATION_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH);
    let budgets = read_runtime_src(DELEGATION_SPLIT_LAYOUT_BUDGETS_CHILD_PATH);
    let split_layout = read_runtime_src(DELEGATION_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH);
    let status_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_BASE_CHILD_OWNER_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_BASE_CHILD_OWNER_DATE_MAP_PATH);

    assert_contains_all(
        "review-guard delegation split-layout mounts focused children",
        &parent,
        &[
            "#[path = \"split/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"split/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"split/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"split/status_current.rs\"]",
            "mod status_current;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_review_guard_row_data_delegation_guard_is_folder_backed",
        "production guard review rows record delegation guard folder-backed split",
        "should stay under its focused delegation guard budget",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "delegation/split_layout.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard delegation split-layout children retain moved checks",
        &format!("{route_mounts}\n{status_current}\n{budgets}\n{split_layout}"),
        &[
            DELEGATION_GUARD_FOLDER_BACKED_GUARD_NAME,
            "runtime_15_review_guard_row_data_delegation_guard_status_is_current",
            "runtime_15_review_guard_row_data_delegation_guard_budgets_are_current",
            DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );

    let status_anchors = [
        DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_NAME,
        DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/split_layout.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/split/route_mounts.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/split/status_current.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/split/budgets.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/split/split_layout.rs",
        DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production guard review rows record delegation split-layout guard split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support status map records delegation split-layout guard split",
        &status_map,
        &[
            DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_NAME,
            DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records delegation split-layout guard split",
        &date_map,
        &[
            DELEGATION_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-06",
        ],
    );

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
            "Frameworks 02 plan",
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
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
        assert_contains_all(label, &source, &status_anchors);
    }
}
