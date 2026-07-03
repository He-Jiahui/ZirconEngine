use super::*;

#[path = "row_count/current_counts.rs"]
mod current_counts;
#[path = "row_count/priority_frontmatter.rs"]
mod priority_frontmatter;

const STALE_COUNT_PROSE_GUARD_NAME: &str =
    "Runtime 15 M3 foundation row-data stale-count prose guard";
const STALE_COUNT_PROSE_GUARD_ID: &str =
    "runtime_15_foundation_row_data_stale_count_prose_guard_static_passed_cargo_deferred";
const PRIORITY_DOC_FRONTMATTER_SYNC_NAME: &str =
    "Runtime 15 M3 foundation row-data priority-doc frontmatter sync";
const PRIORITY_DOC_FRONTMATTER_SYNC_ID: &str =
    "runtime_15_foundation_row_data_priority_doc_frontmatter_sync_static_passed_cargo_deferred";
const ROW_COUNT_CHILD_FRONTMATTER_PATH: &str =
    "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count.rs";
const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_FRONTMATTER_PATH: &str =
    "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs";
const STATUS_SUPPORT_STATUS_MAP_FRONTMATTER_PATH: &str =
    "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
const STATUS_SUPPORT_DATE_MAP_FRONTMATTER_PATH: &str =
    "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

const ROW_COUNT_ROUTE_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count.rs";
const CURRENT_COUNTS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count/current_counts.rs";
const PRIORITY_FRONTMATTER_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count/priority_frontmatter.rs";

const ROW_COUNT_CHILD_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 foundation row-data row-count child split";
const ROW_COUNT_CHILD_SPLIT_STATUS_ID: &str =
    "runtime_15_foundation_row_data_row_count_child_split_static_passed_cargo_deferred";

const ROW_COUNT_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "current_counts",
        CURRENT_COUNTS_PATH,
        "runtime_15_foundation_row_data_docs_record_current_row_count",
        &[
            "ROW_COUNT_SYNC_NAME",
            "ROW_COUNT_SYNC_ID",
            "21/23/18/11",
            "73",
            "stale_count",
        ],
    ),
    (
        "priority_frontmatter",
        PRIORITY_FRONTMATTER_PATH,
        "runtime_15_foundation_row_data_priority_doc_frontmatter_records_stale_count_guard",
        &[
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "PRIORITY_DOC_FRONTMATTER_SYNC_NAME",
            "PRIORITY_DOC_FRONTMATTER_SYNC_ID",
            "priority_doc_frontmatter",
        ],
    ),
];

#[test]
fn runtime_15_foundation_row_data_row_count_children_are_child_owned() {
    let route_source = read_runtime_src(ROW_COUNT_ROUTE_PATH);

    for (module_name, path, guard_name, labels) in ROW_COUNT_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "foundation row-data row-count route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_source, labels);

        let line_count = child_source.lines().count();
        assert!(
            line_count < 140,
            "{path} should stay below its focused row-count child budget; got {line_count} lines"
        );
    }
    assert!(
        !route_source.contains("let foundation_core_rows ="),
        "row_count.rs should delegate row-count source reads to child files"
    );
}

#[test]
fn runtime_15_foundation_row_data_row_count_child_split_is_status_recorded() {
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        ROW_COUNT_CHILD_SPLIT_STATUS_NAME,
        ROW_COUNT_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count/current_counts.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data_status_docs/row_count/priority_frontmatter.rs",
        "runtime_15_foundation_row_data_row_count_children_are_child_owned",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records foundation row-count child split",
        &production_guard_support,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status map records foundation row-count child split",
        &status_map,
        &[
            ROW_COUNT_CHILD_SPLIT_STATUS_NAME,
            ROW_COUNT_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 date map records foundation row-count child split",
        &date_map,
        &[ROW_COUNT_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
