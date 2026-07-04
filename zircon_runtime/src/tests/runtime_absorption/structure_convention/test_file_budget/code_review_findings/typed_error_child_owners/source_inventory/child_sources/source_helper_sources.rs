use super::super::super::super::super::*;
use super::super::*;

pub(super) fn typed_error_source_inventory_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_CHILDREN
        .iter()
        .chain(TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN.iter())
        .chain(TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILDREN.iter())
        .chain(TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN.iter())
        .chain(TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN.iter())
        .chain(TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN.iter())
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_source_inventory_child_source_blob() -> String {
    super::source_blobs::source_blob_from(typed_error_source_inventory_child_sources())
}

pub(super) fn typed_error_source_inventory_source_helper_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN
        .iter()
        .chain(TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILDREN.iter())
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_source_inventory_source_helper_child_source_blob() -> String {
    super::source_blobs::source_blob_from(typed_error_source_inventory_source_helper_child_sources())
}

pub(super) fn typed_error_source_inventory_child_inventory_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_source_inventory_child_inventory_child_source_blob() -> String {
    super::source_blobs::source_blob_from(
        typed_error_source_inventory_child_inventory_child_sources(),
    )
}
