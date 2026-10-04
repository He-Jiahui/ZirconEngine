//! 为类型化错误审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::super::super::*;
use super::*;

pub(in super::super) fn typed_error_native_plugin_loader_route_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_NATIVE_STRUCTURE_ROUTE_CHILDREN
        .iter()
        .chain(TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_CHILDREN.iter())
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_native_plugin_loader_route_child_source_blob() -> String {
    source_blob_from(typed_error_native_plugin_loader_route_child_sources())
}

pub(in super::super) fn typed_error_native_plugin_loader_route_source_helper_child_sources(
) -> Vec<(&'static str, String)> {
    TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in super::super) fn typed_error_native_plugin_loader_route_source_helper_child_source_blob(
) -> String {
    source_blob_from(typed_error_native_plugin_loader_route_source_helper_child_sources())
}

fn source_blob_from(sources: Vec<(&'static str, String)>) -> String {
    let mut blob = String::new();
    for (_, source) in sources {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
