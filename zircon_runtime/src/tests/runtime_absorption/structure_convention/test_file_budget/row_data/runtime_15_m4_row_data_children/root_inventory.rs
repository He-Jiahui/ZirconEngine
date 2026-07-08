use super::*;

#[test]
fn runtime_15_m4_row_data_children_root_inventory_is_child_owned() {
    let parent = read_runtime_src(M4_ROW_DATA_CHILDREN_GUARD_PATH);
    let status_rows = read_runtime_src(RUNTIME_15_M4_ROW_DATA_OWNER_PATH);
    let status_map = read_runtime_src(M4_STATUS_MAP_PATH);
    let date_map = read_runtime_src(M4_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "Runtime 15 M4 row-data children root mounts inventory children",
        &parent,
        &[
            "#[path = \"runtime_15_m4_row_data_children/root_paths.rs\"]",
            "#[path = \"runtime_15_m4_row_data_children/root_statuses.rs\"]",
            "#[path = \"runtime_15_m4_row_data_children/root_child_rows.rs\"]",
            "#[path = \"runtime_15_m4_row_data_children/root_owner_paths.rs\"]",
            "#[path = \"runtime_15_m4_row_data_children/root_inventory.rs\"]",
        ],
    );

    let status_anchors = [
        ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
        ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_statuses.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_child_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_owner_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_inventory.rs",
        ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("Runtime 15 M4 row-data owner rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M4 status map records Runtime 15 M4 row-data children root inventory split",
        &status_map,
        &[
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M4 date map records Runtime 15 M4 row-data children root inventory split",
        &date_map,
        &[ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
