use super::super::super::super::*;
use super::*;

#[path = "child_sources/delegation_sources.rs"]
mod delegation_sources;
#[path = "child_sources/root_sources.rs"]
mod root_sources;
#[path = "child_sources/source_blobs.rs"]
mod source_blobs;
#[path = "child_sources/source_helper_sources.rs"]
mod source_helper_sources;
#[path = "child_sources/structure_guard.rs"]
mod structure_guard;

pub(super) use delegation_sources::*;
pub(super) use root_sources::*;
pub(super) use source_blobs::*;
pub(super) use source_helper_sources::*;

pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_sources",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_ROOT_SOURCES_CHILD,
        "pub(super) struct TypedErrorSourceInventorySources",
    ),
    (
        "source_helper_sources",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_SOURCE_HELPER_SOURCES_CHILD,
        "pub(super) fn typed_error_source_inventory_source_helper_child_sources",
    ),
    (
        "delegation_sources",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_DELEGATION_SOURCES_CHILD,
        "pub(super) fn typed_error_source_inventory_delegation_child_sources",
    ),
    (
        "source_blobs",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_SOURCE_BLOBS_CHILD,
        "pub(super) fn source_blob_from",
    ),
    (
        "structure_guard",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILD,
        "TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILDREN",
    ),
];

#[test]
fn runtime_15_typed_error_source_inventory_child_sources_are_folder_backed() {
    structure_guard::assert_typed_error_source_inventory_child_sources_are_folder_backed();
}
