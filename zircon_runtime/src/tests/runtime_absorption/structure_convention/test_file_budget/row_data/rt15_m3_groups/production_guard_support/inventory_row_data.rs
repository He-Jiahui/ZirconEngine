use super::*;

pub(super) fn assert_runtime_15_m3_child_groups_inventory_row_data_is_child_owned() {
    let parent = read_runtime_src(
        PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_ROWS_PATH,
    );
    let core_and_evidence = read_runtime_src(PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_ROWS_PATH);
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let runtime_15_m3 = [
        read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support_exports.rs",
        ),
    ]
    .join("\n");
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let status_rows = read_runtime_src(
        PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_GUARD_ROWS_PATH,
    );
    let status_map = [
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_CHILD_GROUP_ROW_DATA_PATH),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_CHILD_GROUP_ROW_DATA_PATH),
    ]
    .join("\n");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (module_name, path, representative_row) in [
        (
            "root_inventory_rows",
            PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_ROOT_ROWS_PATH,
            "Runtime 15 M3 child-groups root inventory child split",
        ),
        (
            "owner_path_rows",
            PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_OWNER_PATH_ROWS_PATH,
            "Runtime 15 M3 child-groups production guard owner-path budget child split",
        ),
        (
            "root_path_rows",
            PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_ROOT_PATH_ROWS_PATH,
            "Runtime 15 M3 child-groups root path constants child split",
        ),
        (
            "guard_inventory_rows",
            PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_GUARD_ROWS_PATH,
            INVENTORY_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
        ),
    ] {
        let path_attr = format!("#[path = \"child_group_inventory_rows/{module_name}.rs\"]");
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "child_group_inventory_rows parent mounts child row module",
            &parent,
            &[path_attr.as_str(), module_mount.as_str()],
        );
        let child = read_runtime_src(path);
        assert_contains_all(path, &child, &[representative_row]);
    }

    let explicit_groups = [
        "CHILD_GROUP_INVENTORY_ROOT_EXPECTED_STATUS_OUTPUT_SLICES",
        "CHILD_GROUP_INVENTORY_OWNER_PATH_EXPECTED_STATUS_OUTPUT_SLICES",
        "CHILD_GROUP_INVENTORY_ROOT_PATH_EXPECTED_STATUS_OUTPUT_SLICES",
        "CHILD_GROUP_INVENTORY_GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES",
    ];
    for source in [
        core_and_evidence.as_str(),
        production_guard_support.as_str(),
        runtime_15_m3.as_str(),
        runtime_15.as_str(),
        top_level.as_str(),
    ] {
        assert_contains_all(
            "expected-status aggregation exports explicit child-group inventory groups",
            source,
            &explicit_groups,
        );
        assert!(
            !source.contains("CHILD_GROUP_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES"),
            "expected-status aggregation should not keep the old combined child-group inventory group"
        );
    }

    let status_anchors = [
        INVENTORY_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
        INVENTORY_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/root_inventory_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/owner_path_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/root_path_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/guard_inventory_rows.rs",
        INVENTORY_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
        "scoped rustfmt/static scans passed",
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("child-group inventory guard rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "status map records child-group inventory row-data split",
        &status_map,
        &[
            INVENTORY_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            INVENTORY_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records child-group inventory row-data split",
        &date_map,
        &[INVENTORY_ROW_DATA_CHILD_SPLIT_STATUS_NAME, "2026-07-05"],
    );
}
