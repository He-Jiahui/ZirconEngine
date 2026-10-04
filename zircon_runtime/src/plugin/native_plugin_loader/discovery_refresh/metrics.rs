//! 记录生成当前发布快照的收集工作量，供发现测试区分全扫、单清单更新和纯索引删除。
//! 计数饱和避免诊断统计溢出；它们不参与资源准入，亦不跨代际累计。
/// Structured filesystem work counters for one published discovery generation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct NativePluginDiscoveryRefreshMetrics {
    pub enumerated_directories: u64,
    pub inspected_entries: u64,
    pub manifests_read: u64,
    pub manifests_parsed: u64,
}

impl NativePluginDiscoveryRefreshMetrics {
    pub(super) fn record_traversal(&mut self, directories: u64, entries: u64) {
        self.enumerated_directories = self.enumerated_directories.saturating_add(directories);
        self.inspected_entries = self.inspected_entries.saturating_add(entries);
    }

    pub(super) fn record_manifest_read(&mut self) {
        self.manifests_read = self.manifests_read.saturating_add(1);
    }

    pub(super) fn record_manifest_parse(&mut self) {
        self.manifests_parsed = self.manifests_parsed.saturating_add(1);
    }
}
