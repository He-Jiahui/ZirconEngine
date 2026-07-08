use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split",
        super::top_level::TYPED_ERROR_STRUCTURE_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure assertions guard child-owner split",
        super::top_level::STRUCTURE_ASSERTIONS_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure source/status-map sync",
        super::top_level::TYPED_ERROR_STRUCTURE_SOURCE_STATUS_MAP_SYNC,
    ),
    (
        "Runtime 15 M3 typed-error structure guard folder-backed split",
        super::folder_backed::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure guard root inventory child split",
        super::folder_backed::STRUCTURE_GUARD_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error child-ownership guard folder-backed split",
        super::child_ownership::CHILD_OWNERSHIP_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error child-ownership root inventory child split",
        super::child_ownership::CHILD_OWNERSHIP_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split",
        super::structure_assertions::STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure assertions source reconciliation",
        super::structure_assertions::STRUCTURE_ASSERTIONS_SOURCE_RECONCILIATION,
    ),
    (
        "Runtime 15 M3 typed-error convergence mounts guard folder-backed split",
        super::structure_assertions::CONVERGENCE_MOUNTS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error convergence mounts root inventory child split",
        super::structure_assertions::CONVERGENCE_MOUNTS_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split",
        super::status_docs::STATUS_DOC_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split",
        super::status_docs::DOC_MIRRORS_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc doc mirrors source helper child split",
        super::status_docs::DOC_MIRRORS_SOURCE_HELPER_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error status-doc source helper child split",
        super::status_docs::STATUS_DOC_SOURCE_HELPER_CHILD_SPLIT,
    ),
];
