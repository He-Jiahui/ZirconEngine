use super::*;

#[test]
fn runtime_15_foundation_guards_row_data_root_inventory_is_child_owned() {
    let parent = read_runtime_src(FOUNDATION_GUARDS_ROW_DATA_GUARD_PATH);
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "foundation-guards row-data root mounts inventory children",
        &parent,
        &[
            "#[path = \"runtime_15_foundation_guards_row_data/root_paths.rs\"]",
            "#[path = \"runtime_15_foundation_guards_row_data/root_statuses.rs\"]",
            "#[path = \"runtime_15_foundation_guards_row_data/root_child_rows.rs\"]",
            "#[path = \"runtime_15_foundation_guards_row_data/root_owner_paths.rs\"]",
            "#[path = \"runtime_15_foundation_guards_row_data/root_inventory.rs\"]",
        ],
    );

    let status_anchors = [
        ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
        ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/root_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/root_statuses.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/root_child_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/root_owner_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_foundation_guards_row_data/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("production guard runtime rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status map records foundation-guards root inventory split",
        &status_map,
        &[
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records foundation-guards root inventory split",
        &date_map,
        &[ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
