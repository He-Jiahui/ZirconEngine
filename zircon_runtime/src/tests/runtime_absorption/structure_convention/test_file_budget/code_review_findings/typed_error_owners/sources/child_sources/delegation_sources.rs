//! 为类型化错误审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::super::*;
use super::super::*;

pub(in super::super) fn typed_error_source_inventory_delegation_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_source_inventory_delegation_child_source_blob() -> String {
    super::source_blobs::source_blob_from(typed_error_source_inventory_delegation_child_sources())
}

pub(in super::super) fn typed_error_source_inventory_delegation_folder_backed_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_source_inventory_delegation_folder_backed_child_source_blob(
) -> String {
    super::source_blobs::source_blob_from(
        typed_error_source_inventory_delegation_folder_backed_child_sources(),
    )
}

pub(in super::super) fn typed_error_source_inventory_delegation_folder_backed_ownership_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_source_inventory_delegation_folder_backed_ownership_child_source_blob(
) -> String {
    super::source_blobs::source_blob_from(
        typed_error_source_inventory_delegation_folder_backed_ownership_child_sources(),
    )
}
