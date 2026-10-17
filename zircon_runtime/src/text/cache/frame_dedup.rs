//! UI 帧内的测量与布局结果去重；每次 begin_frame 都丢弃旧值，跨帧复用由有容量上限的缓存负责。
//!
//! # 所有权与边界
//! 调用方划定帧边界并提供请求键；本层持有原文和结果，不设容量淘汰上限。
//! 完整原文按 str 相等比较，不做 Unicode 归一化；参见同级共享索引与 UI 文本缓存调用方。
use std::{hash::Hash, sync::Arc};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

/// 帧内查询与显式清理的值快照；事件与候选计数均饱和累计，开始帧时整体重置。
/// 候选访问数是索引工作计量，不表示哈希探针、分配次数或恒定耗时。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextFrameDedupReport {
    /// 最近一次开始帧传入的值，原样保存而不验证递增。
    pub(crate) frame_index: u64,
    /// 导出报告时重新读取的当前条目数。
    pub(crate) entry_count: usize,
    /// 查询及惰性查询命中的次数，不包含显式插入。
    pub(crate) hit_count: u64,
    /// 查询及惰性查询最终未命中的次数。
    pub(crate) miss_count: u64,
    /// 同键曾遇异原文且最终未命中的查询次数；一次查询最多加一。
    pub(crate) collision_miss_count: u64,
    /// 包含显式插入在内，实际到达的存活候选总数。
    pub(crate) lookup_candidate_count: u64,
    /// 显式或惰性入口实际新建条目的次数。
    pub(crate) insert_count: u64,
    /// 显式插入替换已有匹配条目结果的次数。
    pub(crate) update_count: u64,
    /// 显式清理的次数；开始帧的清空不累计此项。
    pub(crate) clear_count: u64,
}

/// 拥有原文的 Arc 和结果；共享索引仅从 key 取分桶身份。
#[derive(Clone, Debug, PartialEq)]
struct TextFrameDedupEntry<K, V> {
    key: K,
    text: Arc<str>,
    value: V,
}

impl<K, V> IndexedTextCacheEntry<K> for TextFrameDedupEntry<K, V> {
    fn cache_key(&self) -> &K {
        &self.key
    }
}

/// 同一帧内按请求键与完整原文共享结果，避免摘要碰撞让不同文本误命中。
///
/// # 使用与生命周期
/// 由 owner 在新帧前调用开始帧入口；重复帧号也会清空。本层不会在帧尾自行清理。
/// 返回的结果引用受当前实例借用约束；克隆复制索引及结果，原文 Arc 可继续共享。
/// 清空条目不保证归还容器容量，也不能释放仍由其他 Arc 持有的原文。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextFrameDedup<K: Eq + Hash, V> {
    /// 不启用 LRU 的共享索引；完整原文匹配由本层谓词负责。
    index: IndexedTextCache<K, TextFrameDedupEntry<K, V>>,
    /// 本帧统计底稿；导出快照时再刷新条目数。
    frame_report: TextFrameDedupReport,
}

impl<K, V> Default for TextFrameDedup<K, V>
where
    K: Clone + Eq + Hash,
{
    /// 构造空索引和全零报告；不会替调用方选择首帧序号。
    fn default() -> Self {
        Self {
            index: IndexedTextCache::new(),
            frame_report: TextFrameDedupReport::default(),
        }
    }
}

impl<K, V> TextFrameDedup<K, V>
where
    K: Clone + Eq + Hash,
{
    /// 无条件开始指定帧，清空条目并重置全部报告统计。
    /// `frame_index` 原样保存，不检查相同或倒退；该清空不累计显式清理次数。
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.index.clear();
        self.frame_report = TextFrameDedupReport {
            frame_index,
            ..TextFrameDedupReport::default()
        };
    }

    /// 清空条目并饱和累计显式清理次数，保留帧号和其余统计。
    /// 调用方仍需按自己的帧边界重置报告；本入口不承诺释放全部内存。
    pub(crate) fn clear(&mut self) {
        self.index.clear();
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.refresh_report_size();
    }

    /// 返回当前存活条目数。
    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }

    /// 返回当前是否没有存活条目。
    pub(crate) fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// 返回独立拥有的报告值，并以当前索引长度刷新条目数。
    /// 该快照不会随后续查询或插入自动变化。
    pub(crate) fn report(&self) -> TextFrameDedupReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report
    }

    /// 借用键及完整原文查询；命中返回借用结果，未命中返回 None。
    /// 经内部查询累计候选及命中/未命中统计；键相同不能替代原文比较。
    pub(crate) fn get(&mut self, key: &K, text: &str) -> Option<&V> {
        let slot = self.find_slot(key, text)?;
        self.index.entry(slot).map(|entry| &entry.value)
    }

    /// 消费键、可转为 Arc 的原文及结果，返回借用当前条目结果的引用。
    /// 精确匹配时只替换结果，否则新建；原文转换在查找前发生。
    /// 累计候选和插入/更新统计，不累计查询命中或未命中。
    pub(crate) fn insert(&mut self, key: K, text: impl Into<Arc<str>>, value: V) -> &V {
        let text = text.into();
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.text.as_ref() == text.as_ref());
        self.record_lookup(lookup.candidate_count);
        let (_, entry, inserted) = self.index.update_or_insert_with(
            lookup.slot.filter(|slot| self.index.entry(*slot).is_some()),
            value,
            false,
            |entry, value| entry.value = value,
            |value| TextFrameDedupEntry { key, text, value },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        } else {
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
        }
        &entry.value
    }

    /// 接收键与产生闭包的拥有权、借用原文查询，返回借用结果和是否新建。
    /// `produce` 仅在新建分支执行；命中不替换结果，也不累计更新次数。
    /// 与普通查询一样累计命中/未命中；新条目拥有原文 Arc，引用受本次借用约束。
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        text: &str,
        produce: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let slot = self
            .find_slot(&key, text)
            .filter(|slot| self.index.entry(*slot).is_some());
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            (),
            false,
            |_, ()| {},
            |()| TextFrameDedupEntry {
                key,
                text: Arc::from(text),
                value: produce(),
            },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        }
        (&entry.value, inserted)
    }

    /// 只在同键候选中比较完整原文；访问候选后记录一次查询结果。
    /// 曾见异原文但随后命中时，不累计碰撞未命中。
    fn find_slot(&mut self, key: &K, text: &str) -> Option<TextCacheSlot> {
        let mut collision_seen = false;
        let lookup = self.index.find_slot(key, |entry| {
            if entry.text.as_ref() == text {
                true
            } else {
                collision_seen = true;
                false
            }
        });
        self.record_lookup(lookup.candidate_count);
        if let Some(slot) = lookup.slot {
            self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
            return Some(slot);
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        if collision_seen {
            self.frame_report.collision_miss_count =
                self.frame_report.collision_miss_count.saturating_add(1);
        }
        None
    }

    /// 累计实际访问的存活候选数；它不度量容器内部探针或数据移动。
    fn record_lookup(&mut self, candidate_count: usize) {
        self.frame_report.lookup_candidate_count = self
            .frame_report
            .lookup_candidate_count
            .saturating_add(candidate_count as u64);
    }

    /// 在显式清空后同步底稿条目数；报告入口仍会重新读取索引长度。
    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.index.len();
    }
}
