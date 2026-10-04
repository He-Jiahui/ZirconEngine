//! 为类型化错误审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::*;

pub(super) fn typed_error_sources() -> Vec<(&'static str, String)> {
    super::paths::typed_error_source_paths()
        .iter()
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_children_source() -> String {
    let mut children = String::new();
    for (path, source) in typed_error_sources() {
        children.push_str(path);
        children.push('\n');
        children.push_str(&source);
        children.push('\n');
    }
    children
}

pub(super) fn typed_error_review_guard_count() -> usize {
    typed_error_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}
