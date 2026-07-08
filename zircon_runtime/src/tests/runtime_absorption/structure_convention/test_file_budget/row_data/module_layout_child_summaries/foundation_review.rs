use super::*;

#[path = "foundation_review/foundation_status_docs.rs"]
mod foundation_status_docs;
#[path = "foundation_review/review_guard_rows.rs"]
mod review_guard_rows;
#[path = "foundation_review/runtime_foundation_rows.rs"]
mod runtime_foundation_rows;

const FOUNDATION_REVIEW_ROUTE_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review.rs";
const RUNTIME_FOUNDATION_ROWS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/runtime_foundation_rows.rs";
const FOUNDATION_STATUS_DOCS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/foundation_status_docs.rs";
const REVIEW_GUARD_ROWS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/review_guard_rows.rs";

const FOUNDATION_REVIEW_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 module-layout child-summary foundation-review child split";
const FOUNDATION_REVIEW_SPLIT_STATUS_ID: &str =
    "runtime_15_module_layout_child_summary_foundation_review_child_split_static_passed_cargo_deferred";

const FOUNDATION_REVIEW_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "runtime_foundation_rows",
        RUNTIME_FOUNDATION_ROWS_PATH,
        "runtime_15_module_layout_child_summary_runtime_foundation_rows_are_child_owned",
        &[
            "evidence anchor child owns variable evidence guard",
            "Runtime 15 row-data child owns Runtime 15 parent split guard",
            "Runtime 15 foundation row-data child owns foundation split guard",
        ],
    ),
    (
        "foundation_status_docs",
        FOUNDATION_STATUS_DOCS_PATH,
        "runtime_15_module_layout_child_summary_foundation_status_docs_are_child_owned",
        &[
            "Runtime 15 foundation row-data status-doc child owns status/doc anchors",
            "Runtime 15 foundation row-data status-doc folder owns status/doc anchors",
        ],
    ),
    (
        "review_guard_rows",
        REVIEW_GUARD_ROWS_PATH,
        "runtime_15_module_layout_child_summary_review_guard_rows_are_child_owned",
        &[
            "Runtime 15 review-guard row-data child owns review-guard split guard",
            "Runtime 15 review-guard row-data moved-row child owns moved-row assertions",
            "Runtime 15 review-guard row-data moved-row folder owns moved-row assertions",
            "Runtime 15 review-guard row-data status-doc child owns status/doc anchors",
        ],
    ),
];

#[test]
fn runtime_15_module_layout_child_summary_foundation_review_rows_are_child_owner() {
    let child_summary_parent = read_runtime_src(CHILD_SUMMARY_PARENT_PATH);
    let route_source = read_runtime_src(FOUNDATION_REVIEW_ROUTE_PATH);

    for (_, _, _, labels) in FOUNDATION_REVIEW_CHILDREN {
        for delegated_summary in *labels {
            assert!(
                !child_summary_parent.contains(delegated_summary),
                "module_layout_child_summaries.rs should delegate {delegated_summary}"
            );
        }
    }

    for (module_name, path, guard_name, labels) in FOUNDATION_REVIEW_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "module-layout child-summary foundation-review route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_source, labels);
    }
    assert!(
        !route_source.contains(concat!("let evidence", "_anchors_parent =")),
        "foundation_review.rs should delegate row/status reads to child files"
    );
}

#[test]
fn runtime_15_module_layout_child_summary_foundation_review_children_are_status_recorded() {
    let production_guard_support =
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH);
    let expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        FOUNDATION_REVIEW_SPLIT_STATUS_NAME,
        FOUNDATION_REVIEW_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/runtime_foundation_rows.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/foundation_status_docs.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/foundation_review/review_guard_rows.rs",
        "runtime_15_module_layout_child_summary_foundation_review_rows_are_child_owner",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records foundation-review child split",
        &production_guard_support,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support map records foundation-review child split",
        &expected_status_map,
        &[
            FOUNDATION_REVIEW_SPLIT_STATUS_NAME,
            FOUNDATION_REVIEW_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 date map records foundation-review child split",
        &expected_date_map,
        &[FOUNDATION_REVIEW_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
