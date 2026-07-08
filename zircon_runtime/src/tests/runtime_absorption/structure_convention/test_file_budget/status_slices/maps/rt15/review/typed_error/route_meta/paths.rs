use super::*;

pub(super) const TYPED_ERROR_ROUTE_SLICE: &str =
    "Runtime 15 M3 review-guard typed-error expected-slice route metadata child split";
pub(super) const TYPED_ERROR_ROUTE_STATUS: &str =
    "runtime_15_review_guard_typed_error_expected_slice_route_metadata_child_split_static_passed_cargo_deferred";
pub(super) const TYPED_ERROR_ROUTE_GUARD: &str =
    "runtime_15_review_guard_expected_slice_typed_error_route_metadata_is_child_owned";

pub(super) const ROUTE_METADATA_SLICE: &str =
    "Runtime 15 M3 review-guard typed-error expected-slice route metadata folder-backed split";
pub(super) const ROUTE_METADATA_STATUS: &str =
    "runtime_15_review_guard_typed_error_expected_slice_route_metadata_folder_backed_static_passed_cargo_deferred";
pub(super) const ROUTE_METADATA_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_review_guard_typed_error_expected_slice_route_metadata_folder_backed_static_passed_cargo_deferred";
pub(super) const ROUTE_METADATA_GUARD: &str =
    "runtime_15_review_guard_expected_slice_typed_error_route_metadata_is_folder_backed";

pub(super) const STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error_expected_slice.rs";
pub(super) const STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/sources.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/guard_body.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/map_rows.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_metadata.rs",
];
pub(super) const ROUTE_METADATA_ROUTE_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_metadata.rs";
pub(super) const ROUTE_METADATA_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_meta/budgets.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_meta/doc_mirrors.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_meta/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_meta/paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_meta/route_mounts.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/route_meta/status_mirrors.rs",
];

pub(super) fn read_route_metadata_children() -> String {
    ROUTE_METADATA_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
