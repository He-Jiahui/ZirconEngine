use super::*;

#[path = "inventory/delegation_children.rs"]
mod delegation_children;
#[path = "inventory/folder_backed_children.rs"]
mod folder_backed_children;
#[path = "inventory/folder_backed_ownership_children.rs"]
mod folder_backed_ownership_children;
#[path = "inventory/root_children.rs"]
mod root_children;
#[path = "inventory/source_helper_children.rs"]
mod source_helper_children;
#[path = "inventory/status_current.rs"]
mod status_current;

pub(super) use delegation_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN;
pub(super) use folder_backed_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN;
pub(super) use folder_backed_ownership_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN;
pub(super) use root_children::TYPED_ERROR_SOURCE_INVENTORY_CHILDREN;
pub(super) use source_helper_children::TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN;

pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation_children",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_DELEGATION_CHILDREN_CHILD,
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN",
    ),
    (
        "folder_backed_children",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_FOLDER_BACKED_CHILDREN_CHILD,
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN",
    ),
    (
        "folder_backed_ownership_children",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_FOLDER_BACKED_OWNERSHIP_CHILDREN_CHILD,
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN",
    ),
    (
        "root_children",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_ROOT_CHILDREN_CHILD,
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_CHILDREN",
    ),
    (
        "source_helper_children",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_SOURCE_HELPER_CHILDREN_CHILD,
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN",
    ),
    (
        "status_current",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CHILD,
        "pub(in super::super) fn assert_typed_error_source_inventory_child_inventory_is_folder_backed",
    ),
];

#[test]
fn runtime_15_typed_error_source_inventory_child_inventory_is_folder_backed() {
    status_current::assert_typed_error_source_inventory_child_inventory_is_folder_backed();
}
