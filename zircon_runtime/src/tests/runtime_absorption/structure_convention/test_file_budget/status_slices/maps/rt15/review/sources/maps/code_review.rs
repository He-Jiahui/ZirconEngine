use super::*;

pub(in super::super::super) const CODE_REVIEW_STATUS_MAP_SOURCE: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/sources/maps/code_review.rs";
pub(in super::super::super) const STATUS_REVIEW_CODE_REVIEW_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review_guard_maps.rs";
pub(in super::super::super) const STATUS_REVIEW_CODE_REVIEW_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/direct_assertion_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/expected_slice_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/folder_backed_summary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/source_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/structure_guard_rows.rs",
];
pub(in super::super::super) const DATE_REVIEW_CODE_REVIEW_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review_guard_maps.rs";
pub(in super::super::super) const DATE_REVIEW_CODE_REVIEW_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/direct_assertion_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/expected_slice_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/folder_backed_summary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/source_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/structure_guard_rows.rs",
];

pub(in super::super::super) fn read_status_review_code_review_sources() -> String {
    read_review_code_review_sources(
        STATUS_REVIEW_CODE_REVIEW_CHILD,
        STATUS_REVIEW_CODE_REVIEW_CHILDREN,
    )
}

pub(in super::super::super) fn read_date_review_code_review_sources() -> String {
    read_review_code_review_sources(
        DATE_REVIEW_CODE_REVIEW_CHILD,
        DATE_REVIEW_CODE_REVIEW_CHILDREN,
    )
}

fn read_review_code_review_sources(parent: &str, children: &[&str]) -> String {
    std::iter::once(parent)
        .chain(children.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
