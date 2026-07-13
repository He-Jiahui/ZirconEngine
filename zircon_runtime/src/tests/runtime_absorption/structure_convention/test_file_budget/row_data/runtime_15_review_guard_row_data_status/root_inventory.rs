use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_docs_root_inventory_is_child_owned() {
    let parent = read_runtime_src(STATUS_DOCS_GUARD_PATH);
    let status_rows = status_support_review_guard_source_blob();
    let status_map = status_support_status_map_source_blob();
    let date_map = status_support_date_map_source_blob();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "review-guard status-doc row root mounts inventory children",
        &parent,
        &[
            "#[path = \"runtime_15_review_guard_row_data_status/root_paths.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data_status/root_statuses.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data_status/root_child_rows.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data_status/root_source_blobs.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data_status/root_inventory.rs\"]",
        ],
    );

    let status_anchors = [
        ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
        ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/root_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/root_statuses.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/root_child_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/root_source_blobs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("production guard review rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status map records review-guard status-doc root inventory split",
        &status_map,
        &[
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records review-guard status-doc root inventory split",
        &date_map,
        &[ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
