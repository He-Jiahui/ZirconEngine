//! 为插件导入接口审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::*;

pub(super) fn plugin_importer_dx_sources() -> Vec<(&'static str, String)> {
    super::paths::plugin_importer_dx_source_paths()
        .iter()
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_dx_review_guard_count() -> usize {
    plugin_importer_dx_sources()
        .iter()
        .map(|(_, source)| {
            source
                .lines()
                .filter(|line| line.trim_start().starts_with("fn review_"))
                .count()
        })
        .sum()
}
