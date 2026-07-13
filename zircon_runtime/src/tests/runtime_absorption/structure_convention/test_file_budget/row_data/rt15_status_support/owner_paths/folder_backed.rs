use super::*;

const STATUS_SUPPORT_ROW_OWNER_PATH_CHILDREN: &[(&str, &str)] = &[
    (
        "expected_slice_maps",
        "STATUS_SUPPORT_EXPECTED_SLICE_MAP_OWNER_PATH_GROUPS",
    ),
    (
        "priority_plan_docs",
        "STATUS_SUPPORT_PRIORITY_PLAN_DOC_OWNER_PATHS",
    ),
    ("root_rows", "STATUS_SUPPORT_ROOT_ROW_OWNER_PATHS"),
    (
        "row_data_and_budget",
        "STATUS_SUPPORT_ROW_DATA_AND_BUDGET_OWNER_PATHS",
    ),
    (
        "runtime_index_anchors",
        "STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_OWNER_PATHS",
    ),
];

fn status_support_root_owner_paths_child_path(module_name: &str) -> String {
    format!(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/{module_name}.rs"
    )
}

#[test]
fn runtime_15_status_support_row_data_root_owner_paths_are_folder_backed() {
    let parent = read_runtime_src(ROOT_OWNER_PATHS_PATH);
    let budgets = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/budgets.rs",
    );
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_SUPPORT_PRIORITY_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (module_name, representative_anchor) in STATUS_SUPPORT_ROW_OWNER_PATH_CHILDREN {
        let module_mount = format!("#[path = \"owner_paths/{module_name}.rs\"]");
        assert_contains_all(
            "status-support root owner paths parent mounts focused child groups",
            &parent,
            &[module_mount.as_str(), &format!("mod {module_name};")],
        );
        assert_contains_all(
            "status-support root owner path child owns representative budget group",
            &read_runtime_src(&status_support_root_owner_paths_child_path(module_name)),
            &[representative_anchor],
        );
    }
    assert_contains_all(
        "status-support root owner paths parent exposes grouped budget traversal",
        &parent,
        &[
            "#[path = \"owner_paths/folder_backed.rs\"]",
            "status_support_row_owner_path_groups",
        ],
    );
    assert_contains_all(
        "status-support row-data budgets traverse grouped owner paths",
        &budgets,
        &["status_support_row_owner_path_groups()"],
    );
    assert!(
        !budgets.contains("STATUS_SUPPORT_ROW_OWNER_PATH_GROUPS"),
        "budgets.rs should use iterator-based status-support row owner path groups"
    );

    for moved_anchor in [
        "STATUS_SUPPORT_ROW_OWNER_PATHS",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "root_owner_paths.rs should route grouped owner paths instead of retaining {moved_anchor}"
        );
    }

    let status_anchors = [
        ROOT_OWNER_PATHS_FOLDER_BACKED_STATUS_NAME,
        ROOT_OWNER_PATHS_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/rt15_status_support/root_owner_paths.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/root_rows.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/row_data_and_budget.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/runtime_index_anchors.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/priority_plan_docs.rs",
        ROOT_OWNER_PATHS_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", frameworks_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("production guard runtime-row rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status-support status map records root owner paths split",
        &status_map,
        &[
            ROOT_OWNER_PATHS_FOLDER_BACKED_STATUS_NAME,
            ROOT_OWNER_PATHS_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records root owner paths split",
        &date_map,
        &[ROOT_OWNER_PATHS_FOLDER_BACKED_STATUS_NAME, "2026-07-06"],
    );
}
