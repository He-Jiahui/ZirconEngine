use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 typed-error status-doc delegation child split",
        super::status_docs::STATUS_DOC_DELEGATION_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc delegation status-current child split",
        super::status_docs::STATUS_DOC_DELEGATION_STATUS_CURRENT_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc delegation status-current sources child split",
        super::status_docs::STATUS_DOC_DELEGATION_STATUS_CURRENT_SOURCES_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc delegation status-current sources guard folder-backed split",
        super::status_docs::STATUS_DOC_DELEGATION_STATUS_CURRENT_SOURCES_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc delegation status-current split-layout guard folder-backed split",
        super::status_docs::STATUS_DOC_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc delegation status-current split-layout sources child split",
        super::status_docs::STATUS_DOC_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_SOURCES_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc delegation status-current split-layout sources guard folder-backed split",
        super::status_docs::STATUS_DOC_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_SOURCES_GUARD_FOLDER_BACKED_SPLIT,
    ),
];
