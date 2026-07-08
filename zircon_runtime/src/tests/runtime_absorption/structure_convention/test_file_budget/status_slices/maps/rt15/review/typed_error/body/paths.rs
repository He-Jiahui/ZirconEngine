use super::*;

pub(super) const GUARD_BODY_SLICE: &str =
    "Runtime 15 M3 review-guard typed-error expected-slice guard body folder-backed split";
pub(super) const GUARD_BODY_STATUS: &str =
    "runtime_15_review_guard_typed_error_expected_slice_guard_body_folder_backed_static_passed_cargo_deferred";
pub(super) const GUARD_BODY_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_review_guard_typed_error_expected_slice_guard_body_folder_backed_static_passed_cargo_deferred";
pub(super) const GUARD_BODY_GUARD: &str =
    "runtime_15_review_guard_expected_slice_typed_error_guard_body_is_folder_backed";

pub(super) const GUARD_BODY_ROUTE_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/guard_body.rs";
pub(super) const GUARD_BODY_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/body/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/body/literal_ownership.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/body/paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/body/route_mounts.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/body/status_docs.rs",
];

pub(super) fn read_guard_body_children() -> String {
    GUARD_BODY_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
