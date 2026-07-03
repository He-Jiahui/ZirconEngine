use super::Slice;

#[path = "typed_error_structure_rows/folder_backed.rs"]
mod folder_backed;
#[path = "typed_error_structure_rows/status_docs.rs"]
mod status_docs;
#[path = "typed_error_structure_rows/structure_assertions.rs"]
mod structure_assertions;
#[path = "typed_error_structure_rows/top_level.rs"]
mod top_level;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split",
        top_level::TYPED_ERROR_STRUCTURE_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure assertions guard child-owner split",
        top_level::STRUCTURE_ASSERTIONS_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure guard folder-backed split",
        folder_backed::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split",
        structure_assertions::STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split",
        status_docs::STATUS_DOC_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split",
        structure_assertions::NATIVE_PLUGIN_LOADER_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split",
        structure_assertions::MOVED_GUARD_ABSENCE_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split",
        structure_assertions::MOVED_GUARD_ABSENCE_GUARD_FOLDER_BACKED_SPLIT,
    ),
];
