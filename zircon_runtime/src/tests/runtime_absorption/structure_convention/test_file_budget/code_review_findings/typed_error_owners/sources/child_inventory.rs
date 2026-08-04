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

pub(super) use delegation_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN;
pub(super) use folder_backed_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN;
pub(super) use folder_backed_ownership_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN;
pub(super) use root_children::TYPED_ERROR_SOURCE_INVENTORY_CHILDREN;
