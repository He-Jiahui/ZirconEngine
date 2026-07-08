#[path = "metadata/child_inventory_paths.rs"]
mod child_inventory_paths;
#[path = "metadata/delegation_paths.rs"]
mod delegation_paths;
#[path = "metadata/review_guard_paths.rs"]
mod review_guard_paths;
#[path = "metadata/root_paths.rs"]
mod root_paths;
#[path = "metadata/status_current.rs"]
mod status_current;
#[path = "metadata/status_slices.rs"]
mod status_slices;

pub(super) use child_inventory_paths::*;
pub(super) use delegation_paths::*;
pub(super) use review_guard_paths::*;
pub(super) use root_paths::*;
pub(super) use status_current::*;
pub(super) use status_slices::*;

pub(super) const TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_ROOT_PATHS_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_CHILD",
    ),
    (
        "child_inventory_paths",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD_INVENTORY_PATHS_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_ROUTE_CHILD",
    ),
    (
        "delegation_paths",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_DELEGATION_PATHS_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_STATUS_CHILD",
    ),
    (
        "status_slices",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_SLICES_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_METADATA_SPLIT",
    ),
    (
        "review_guard_paths",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_REVIEW_GUARD_PATHS_CHILD,
        "REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH",
    ),
    (
        "status_current",
        TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILDREN",
    ),
];

#[test]
fn runtime_15_typed_error_source_inventory_metadata_is_child_backed() {
    status_current::assert_typed_error_source_inventory_metadata_is_child_backed();
}
