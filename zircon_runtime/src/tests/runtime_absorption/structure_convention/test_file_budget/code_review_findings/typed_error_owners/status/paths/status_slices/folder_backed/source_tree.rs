use super::super::super::super::super::super::super::*;
use super::super::super::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_SLICES_CHILD;

pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/folder_backed.rs";

const TYPED_ERROR_STATUS_DOCS_STATUS_SLICE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/core.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/paths.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/delegation.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/status_maps.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/status_mirrors.rs",
    TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILD,
];

const TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/folder_backed/source_tree.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/folder_backed/status_current.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/status_slices/folder_backed/folder_backed.rs",
];

pub(super) fn typed_error_status_doc_status_slices_source_blob() -> String {
    let mut blob = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_SLICES_CHILD);
    blob.push('\n');
    for child_path in typed_error_status_doc_status_slices_child_source_paths() {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    for child_path in typed_error_status_doc_status_slices_folder_backed_guard_children() {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

pub(super) fn typed_error_status_doc_status_slices_folder_backed_guard_blob() -> String {
    let mut blob = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILD);
    blob.push('\n');
    for child_path in typed_error_status_doc_status_slices_folder_backed_guard_children() {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

pub(super) fn typed_error_status_doc_status_slices_child_source_paths() -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_STATUS_SLICE_CHILDREN
}

pub(super) fn typed_error_status_doc_status_slices_folder_backed_guard_children(
) -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_STATUS_SLICES_FOLDER_BACKED_CHILDREN
}
