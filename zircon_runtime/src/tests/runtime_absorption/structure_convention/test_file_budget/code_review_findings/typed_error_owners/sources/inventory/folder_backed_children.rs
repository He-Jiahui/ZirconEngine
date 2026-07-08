use super::super::*;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "child_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD_OWNERSHIP_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_GUARD,
    ),
    (
        "guard_body",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_GUARD_BODY_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_guard_is_folder_backed",
    ),
    (
        "status_current",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_STATUS_CHILD,
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_STATUS_GUARD,
    ),
];
