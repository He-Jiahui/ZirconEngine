use super::super::super::*;

#[path = "direct_review_assertions/child_ownership.rs"]
mod child_ownership;
#[path = "direct_review_assertions/delegation.rs"]
mod delegation;
#[path = "direct_review_assertions/f12.rs"]
mod f12;
#[path = "direct_review_assertions/f8.rs"]
mod f8;
#[path = "direct_review_assertions/p0.rs"]
mod p0;
#[path = "direct_review_assertions/render.rs"]
mod render;
#[path = "direct_review_assertions/root_parent.rs"]
mod root_parent;
#[path = "direct_review_assertions/status_mirrors.rs"]
mod status_mirrors;

use super::source_inventory::CodeReviewFindingsSources;

pub(super) const FOLDER_BACKED_SUMMARY_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/delegation.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_STATUS_MIRRORS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/status_mirrors.rs";
pub(super) const F12_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs";
pub(super) const F8_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs";
pub(super) const P0_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs";
pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DIRECT_ASSERTION_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/code_review_guard_maps/direct_assertion_rows.rs";
pub(super) const REVIEW_GUARD_DIRECT_ASSERTION_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/code_review_guard_maps/direct_assertion_rows.rs";

pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;
pub(super) const DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings direct assertions guard folder-backed split";
pub(super) const DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_ID: &str = "runtime_15_code_review_findings_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";

pub(super) const DIRECT_REVIEW_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        DIRECT_REVIEW_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_direct_assertions_are_child_owner",
    ),
    (
        "child_ownership",
        DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD,
        "runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
    ),
    (
        "f12",
        F12_DIRECT_ASSERTIONS_CHILD,
        "runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
    ),
    (
        "f8",
        F8_DIRECT_ASSERTIONS_CHILD,
        "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
    ),
    (
        "p0",
        P0_DIRECT_ASSERTIONS_CHILD,
        "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
    ),
    (
        "render",
        RENDER_DIRECT_ASSERTIONS_CHILD,
        "runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
    ),
    (
        "root_parent",
        ROOT_PARENT_DIRECT_ASSERTIONS_CHILD,
        "runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
    ),
    (
        "status_mirrors",
        DIRECT_REVIEW_ASSERTIONS_STATUS_MIRRORS_CHILD,
        "runtime_15_code_review_findings_direct_assertions_guard_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_code_review_direct_sources_are_folder_backed(
    sources: &CodeReviewFindingsSources,
) {
    f12::assert_f12_direct_sources_are_folder_backed(sources);
    f8::assert_f8_direct_sources_are_folder_backed(sources);
    p0::assert_p0_direct_sources_are_folder_backed(sources);
    render::assert_render_direct_sources_are_folder_backed(sources);
    root_parent::assert_code_review_root_parent_is_folder_backed(sources);
}

pub(super) fn direct_review_assertion_child_sources() -> Vec<(&'static str, String)> {
    DIRECT_REVIEW_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn direct_review_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in direct_review_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob.push_str(&child_ownership::direct_assertion_child_ownership_child_source_blob());
    blob.push('\n');
    blob
}

pub(super) fn direct_review_status_rows_source() -> String {
    let mut source = String::new();
    for path in [
        REVIEW_GUARD_STATUS_ROWS_PATH,
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/core_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/f12_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/f8_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/p0_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/render_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/root_parent_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows/row_data_owner_rows.rs",
    ] {
        source.push_str(&read_runtime_src(path));
        source.push('\n');
    }
    source
}

pub(super) fn direct_review_status_map_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_DIRECT_ASSERTION_STATUS_MAP_PATH)
    )
}

pub(super) fn direct_review_date_map_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_DIRECT_ASSERTION_DATE_MAP_PATH)
    )
}
