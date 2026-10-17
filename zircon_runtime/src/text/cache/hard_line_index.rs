//! 按文档键保留规范硬行的源范围，供不同 viewport 复用完整索引。
//!
//! 请求窗口使用硬行序号；返回范围使用同一输入的 UTF-8 字节偏移。
//! 驻留预算约束索引的收集与保留，完整扫描及返回窗口不因此获得峰值内存保证。

use std::{mem::size_of, ops::Range, sync::Arc};

use crate::text::{visit_hard_lines, EphemeralCacheHash, HardLine};

/// 默认驻留条目上限；返回窗口和外部共享引用不计入条目数。
pub(crate) const DEFAULT_HARD_LINE_INDEX_CACHE_CAPACITY: usize = 16;
/// 默认 owner 驻留估计字节上限，不代表整个管线的内存总量。
pub(crate) const DEFAULT_HARD_LINE_INDEX_CACHE_MAX_BYTES: usize = 32 * 1024 * 1024;

/// 调用方提供的文档身份；本类型不自动推进修订，也不验证编号来源。
/// 同键查找仍需共享指针或精确原文验证，避免旧索引被异源复用。
/// Stable owner identity and revision supplied by a retained text document.
///
/// A key is valid only while its owner advances `revision` for every source change.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub(crate) struct TextDocumentKey {
    owner: u64,
    revision: u64,
}

impl TextDocumentKey {
    /// 原样组合调用方的拥有者编号与修订号。
    ///
    /// # 参数与返回
    /// `owner`、`revision` 不作取值限制；返回可复制的键，不持有源文本。
    pub(crate) const fn new(owner: u64, revision: u64) -> Self {
        Self { owner, revision }
    }

    pub(crate) const fn owner(self) -> u64 {
        self.owner
    }

    pub(crate) const fn revision(self) -> u64 {
        self.revision
    }

    /// 为临时缓存查找生成摘要；原始拥有者编号和修订号仍是键的组成。
    pub(crate) fn ephemeral_hash(self) -> EphemeralCacheHash {
        EphemeralCacheHash::from_hashable(&self)
    }
}

/// 保存报告中的有效条目上限与估计字节上限；该值自身不执行准入或归一。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HardLineIndexCacheBudgetSnapshot {
    pub(crate) max_entries: usize,
    pub(crate) max_bytes: usize,
}

/// 读取时填入当前驻留尺寸与有效上限，其他计数为饱和累计历史。
/// 清空条目只刷新尺寸，不重置这些历史计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HardLineIndexCacheReport {
    pub(crate) budget: HardLineIndexCacheBudgetSnapshot,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) build_count: u64,
    pub(crate) source_pointer_hit_count: u64,
    pub(crate) source_exact_compare_count: u64,
    pub(crate) stale_source_alias_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) oversized_bypass_count: u64,
    pub(crate) unkeyed_bypass_count: u64,
}

/// 拥有源 Arc 和完整硬行范围；估计量包含行向量容量，窗口另行复制范围。
#[derive(Clone, Debug, PartialEq)]
struct HardLineIndexEntry {
    key: TextDocumentKey,
    source: Arc<str>,
    lines: Vec<HardLine>,
    estimated_bytes: usize,
    last_access: u64,
}

/// 为不同窗口保留同一文档的完整索引。
///
/// # 所有权与限制
/// 克隆复制条目、行向量和统计状态，源 Arc 仍共享；各副本分别管理驻留。
/// 上限约束本 owner 的估计驻留，不涵盖外部引用和返回窗口。
/// Bounded source-offset index shared by distinct viewport layouts of one retained document.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HardLineIndexCache {
    entries: Vec<HardLineIndexEntry>,
    capacity: usize,
    max_bytes: usize,
    estimated_bytes: usize,
    next_access: u64,
    report: HardLineIndexCacheReport,
}

impl Default for HardLineIndexCache {
    /// 使用默认限值创建空 owner，初始条目、访问序号与累计计数均为零。
    fn default() -> Self {
        Self::with_limits(
            DEFAULT_HARD_LINE_INDEX_CACHE_CAPACITY,
            DEFAULT_HARD_LINE_INDEX_CACHE_MAX_BYTES,
        )
    }
}

impl HardLineIndexCache {
    /// 初始化空缓存，并归一有效限值。
    ///
    /// # 参数与返回
    /// `capacity` 至少取 1，`max_bytes` 至少取一个硬行记录的大小。
    /// 返回独立 owner；条目头、源字节及行向量容量仍合计估算，可能没有条目可驻留。
    ///
    /// # 生命周期
    /// 成功驻留的条目以 Arc 延长源寿命；仅返回范围不延长源文本寿命。
    pub(crate) fn with_limits(capacity: usize, max_bytes: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
            max_bytes: max_bytes.max(size_of::<HardLine>()),
            estimated_bytes: 0,
            next_access: 0,
            report: HardLineIndexCacheReport::default(),
        }
    }

    /// 删除驻留条目并清估计字节；保留累计计数与访问序号。
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    /// 复制报告，并以当前限值、条目数和估计字节覆盖快照中的尺寸字段。
    pub(crate) fn report(&self) -> HardLineIndexCacheReport {
        let mut report = self.report;
        report.budget = HardLineIndexCacheBudgetSnapshot {
            max_entries: self.capacity,
            max_bytes: self.max_bytes,
        };
        report.entry_count = self.entries.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    /// 返回完整硬行总数与指定行序号窗口，并尝试保留完整索引。
    ///
    /// # 参数与返回
    /// `key` 缩小文档查找范围；`source` 移入共享源引用，命中仍验证其指针或原文。
    /// `range` 是从零开始的半开硬行序号范围，不是源字节范围。
    /// 返回的硬行范围为同一源的 UTF-8 字节偏移，空或反向窗口不产生行。
    ///
    /// # 所有权与约束
    /// 返回窗口只拥有范围值，使用这些偏移切片时仍须保留对应源文本。
    /// 同键异源先移除旧项再重建；超估计上限仍扫描并返回窗口，但不保留索引。
    /// 因此驻留限值不限制返回窗口大小，也不保证整个调用的分配峰值。
    ///
    /// # 参考
    /// 规范分行由 [visit_hard_lines] 生成；会话入口另行选择有键缓存或无键旁路。
    pub(crate) fn count_and_window(
        &mut self,
        key: TextDocumentKey,
        source: Arc<str>,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        let access = self.next_access();
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            let source_matches = if Arc::ptr_eq(&self.entries[index].source, &source) {
                self.report.source_pointer_hit_count =
                    self.report.source_pointer_hit_count.saturating_add(1);
                true
            } else {
                self.report.source_exact_compare_count =
                    self.report.source_exact_compare_count.saturating_add(1);
                self.entries[index].source == source
            };
            if source_matches {
                self.report.hit_count = self.report.hit_count.saturating_add(1);
                let entry = &mut self.entries[index];
                entry.last_access = access;
                return (entry.lines.len(), line_window(&entry.lines, range));
            }

            let stale = self.entries.swap_remove(index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(stale.estimated_bytes);
            self.report.stale_source_alias_count =
                self.report.stale_source_alias_count.saturating_add(1);
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let mut lines = Vec::new();
        let mut window = Vec::new();
        let mut line_count = 0usize;
        let max_cached_lines = self.max_cached_line_count(source.len());
        let mut cacheable = true;
        visit_hard_lines(source.as_ref(), |line| {
            if range.contains(&line_count) {
                window.push(line.clone());
            }
            if cacheable {
                cacheable = reserve_hard_line_slot(&mut lines, max_cached_lines);
                if cacheable {
                    lines.push(line);
                } else {
                    lines.clear();
                }
            }
            line_count = line_count.saturating_add(1);
        });

        if !cacheable || estimated_entry_bytes(&source, &lines) > self.max_bytes {
            self.report.oversized_bypass_count =
                self.report.oversized_bypass_count.saturating_add(1);
            return (line_count, window);
        }

        let estimated_bytes = estimated_entry_bytes(&source, &lines);
        self.evict_for(estimated_bytes);
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.entries.push(HardLineIndexEntry {
            key,
            source,
            lines,
            estimated_bytes,
            last_access: access,
        });
        self.report.build_count = self.report.build_count.saturating_add(1);
        self.refresh_report_size();
        (line_count, window)
    }

    /// 插入前按条目数与估计量回收最小访问序号的项；饱和后序号可能并列。
    fn evict_for(&mut self, incoming_bytes: usize) {
        while self.entries.len() >= self.capacity
            || self.estimated_bytes.saturating_add(incoming_bytes) > self.max_bytes
        {
            let Some((oldest_index, _)) = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.last_access)
            else {
                break;
            };
            let entry = self.entries.swap_remove(oldest_index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(entry.estimated_bytes);
            self.report.evicted_count = self.report.evicted_count.saturating_add(1);
        }
    }

    /// 饱和递增访问序号；耗尽后不再形成严格的先后顺序。
    fn next_access(&mut self) -> u64 {
        self.next_access = self.next_access.saturating_add(1);
        self.next_access
    }

    /// 扣除条目头与源字节后，按硬行记录大小估算可驻留行槽，减法均饱和。
    fn max_cached_line_count(&self, source_len: usize) -> usize {
        self.max_bytes
            .saturating_sub(size_of::<HardLineIndexEntry>())
            .saturating_sub(source_len)
            / size_of::<HardLine>()
    }

    /// 只刷新当前条目数与估计量，不修改累计统计。
    fn refresh_report_size(&mut self) {
        self.report.entry_count = self.entries.len();
        self.report.estimated_bytes = self.estimated_bytes;
    }

    /// 为无文档键的调用记录一次旁路；此处不扫描源或构建索引。
    pub(crate) fn record_unkeyed_bypass(&mut self) {
        self.report.unkeyed_bypass_count = self.report.unkeyed_bypass_count.saturating_add(1);
    }
}

/// 达到行数限制时返回 false；需要容量时按受限翻倍请求行向量扩容。
fn reserve_hard_line_slot(lines: &mut Vec<HardLine>, max_lines: usize) -> bool {
    if lines.len() >= max_lines {
        return false;
    }
    if lines.len() == lines.capacity() {
        let current_capacity = lines.capacity();
        let next_capacity = current_capacity.max(1).saturating_mul(2).min(max_lines);
        lines.reserve_exact(next_capacity.saturating_sub(current_capacity));
    }
    true
}

/// 将窗口上界夹到实际行数并复制范围值；空或反向范围返回空向量。
fn line_window(lines: &[HardLine], range: Range<usize>) -> Vec<HardLine> {
    let start = range.start.min(lines.len());
    let end = range.end.min(lines.len());
    (start < end)
        .then(|| lines[start..end].to_vec())
        .unwrap_or_default()
}

/// 按行向量容量、条目头及源字节长度估算驻留。
/// 不计 Arc 控制块、外层 entries 容量或返回窗口，因此不是分配器精确用量。
fn estimated_entry_bytes(source: &Arc<str>, lines: &Vec<HardLine>) -> usize {
    lines
        .capacity()
        .saturating_mul(size_of::<HardLine>())
        .saturating_add(size_of::<HardLineIndexEntry>())
        .saturating_add(source.len())
}

#[cfg(test)]
#[path = "tests/hard_line_index.rs"]
mod tests;
