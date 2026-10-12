//! 为审查目录结构读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::*;

pub(super) fn structure_guard_child_sources() -> Vec<(&'static str, String)> {
    STRUCTURE_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn structure_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in structure_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob.push_str(&budgets::structure_guard_children_budget_child_source_blob());
    blob.push_str(&folder_backed_summary::folder_backed_summary_structure_child_source_blob());
    blob.push_str(&plugin_importer::plugin_importer_structure_guard_child_source_blob());
    blob.push_str(&typed_error::typed_error_structure_guard_child_source_blob());
    blob
}
