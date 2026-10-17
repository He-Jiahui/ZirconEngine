//! UI 文本测量的跨帧驻留缓存；摘要只定位候选，完整原文仍须逐项核对。
//! UI owner 在帧内去重未命中后访问本缓存；泛型层管理键、原文和值，可复用状态由调用方判断。
//! 条目跨帧驻留，读取命中和更新会调整索引的 LRU；帧边界仅整理容量与报告。
//! Arc 原文可由调用方克隆后继续持有；淘汰只释放缓存自己的所有权。
use std::{hash::Hash, sync::Arc};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

pub(crate) const DEFAULT_TEXT_MEASURE_CACHE_CAPACITY: usize = 4096;

/// 当前帧的逻辑操作计数和驻留估算快照。
///
/// # 计量口径
/// 字节估算包含条目本体、文本长度和显式补计值，不代表分配器用量或进程内存上限。
/// 查询计数由读取入口累计；插入只累计候选访问及插入/更新，不能用命中数代替全部访问数。
/// 峰值从帧开始时的驻留估算起算；清空仍保留本帧已出现的峰值和其他历史计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextMeasureCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) peak_estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) lookup_candidate_count: u64,
    pub(crate) eviction_scan_count: u64,
    pub(crate) entry_move_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

/// 索引持有的测量记录；单项估算用于替换、淘汰时扣除，原文另以 Arc 共享。
#[derive(Clone, Debug, PartialEq)]
struct TextMeasureCacheEntry<K, V> {
    key: K,
    text: Arc<str>,
    value: V,
    estimated_bytes: usize,
}

impl<K, V> IndexedTextCacheEntry<K> for TextMeasureCacheEntry<K, V> {
    fn cache_key(&self) -> &K {
        &self.key
    }
}

/// 保存成功测量及其共享原文；UI 帧内缓存借用这份原文，避免命中时再次分配。
///
/// # 使用与生命周期
/// 调用方将影响结果的代次等状态编码进 K，并在写入前确认 V 可复用；本类型不检查结果状态。
/// 返回的值借用缓存；原文的 Arc 只有显式克隆后才取得独立于该借用的持有权。
///
/// # 容量与计量
/// 容量限制驻留条目数，最小为 1；字节字段只是估算遥测，不约束外部克隆或任意堆数据。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextMeasureCache<K: Eq + Hash, V> {
    index: IndexedTextCache<K, TextMeasureCacheEntry<K, V>>,
    capacity: usize,
    estimated_bytes: usize,
    frame_report: TextMeasureCacheReport,
}

impl<K, V> Default for TextMeasureCache<K, V>
where
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> TextMeasureCache<K, V>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn new() -> Self {
        Self::with_capacity(DEFAULT_TEXT_MEASURE_CACHE_CAPACITY)
    }

    /// 创建空缓存，设置逻辑条目上限；不按该数预留索引内存。
    ///
    /// # 参数与返回
    /// capacity 为驻留条目数，0 按 1 处理；返回尚无条目、计数为零的 owner。
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut cache = Self {
            index: IndexedTextCache::new(),
            capacity,
            estimated_bytes: 0,
            frame_report: TextMeasureCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache
    }

    /// 开始一帧的统计，保留驻留条目及其 LRU 次序。
    ///
    /// frame_index 仅写入报告，不验证单调性；其余帧计数归零，峰值以当前驻留估算为起点。
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.frame_report = TextMeasureCacheReport {
            frame_index,
            capacity: self.capacity,
            entry_count: self.index.len(),
            estimated_bytes: self.estimated_bytes,
            peak_estimated_bytes: self.estimated_bytes,
            ..TextMeasureCacheReport::default()
        };
    }

    /// 帧尾只裁掉超过容量的条目，不因本帧未被引用而清空驻留值。
    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_capacity();
    }

    /// 释放全部索引条目，将移除数和清空次数累入本帧报告。
    ///
    /// 当前驻留估算归零；既有峰值及其他计数保留，不增加容量裁剪的工作计数。
    pub(crate) fn clear(&mut self) {
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(self.index.len() as u64);
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.index.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// 复制本帧报告，并以当前索引刷新条目数和驻留估算；不修改缓存状态。
    pub(crate) fn report(&self) -> TextMeasureCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    /// 只检查同键候选中是否存在完整原文相等的条目。
    ///
    /// key 定位候选，text 必须逐字节相等；返回是否存在，不调整 LRU 或任何报告计数。
    pub(crate) fn contains_exact(&self, key: &K, text: &str) -> bool {
        self.index
            .find_slot(key, |entry| entry.text.as_ref() == text)
            .slot
            .is_some()
    }

    /// 查询并借用值，命中会更新 LRU 与读取报告；原文及借用约束见 Self::get_with_stored_text。
    pub(crate) fn get(&mut self, key: &K, text: &str) -> Option<&V> {
        self.get_with_stored_text(key, text).map(|(_, value)| value)
    }

    /// 命中后把驻留的 Arc 原文交还给 UI owner，使本帧去重沿用同一份文本。
    ///
    /// # 参数与返回
    /// key 定位候选，text 用完整原文排除摘要别名；未命中返回 None。
    /// 成功返回的是对 Arc 容器和值的借用，不能越过后续缓存可变借用；需要延寿时由调用方克隆 Arc。
    pub(crate) fn get_with_stored_text(&mut self, key: &K, text: &str) -> Option<(&Arc<str>, &V)> {
        let slot = self.find_slot(key, text)?;
        self.index
            .entry(slot)
            .map(|entry| (&entry.text, &entry.value))
    }

    /// 接收键、共享原文和值，按同键同原文更新或插入，返回缓存内值的借用。
    ///
    /// 本入口不补计额外堆字节；完整更新与计量约束见 Self::insert_with_additional_heap_bytes。
    pub(crate) fn insert(&mut self, key: K, text: impl Into<Arc<str>>, value: V) -> &V {
        self.insert_with_additional_heap_bytes(key, text, value, 0)
    }

    /// 调用方补计 K 和 V 的堆数据；缓存只自动计入条目本体与原文，字节报告用于预算观察。
    ///
    /// # 参数与返回
    /// key、text、value 转交缓存；additional_heap_bytes 由调用方估算键和值另有的堆数据。
    /// 返回更新后值的借用。匹配同键及完整原文时替换值与单项估算，保留原键及已驻留的 Arc。
    ///
    /// # 约束与顺序
    /// 新条目先腾出一个容量位置，再计入估算；原文转换先于查找，不承诺调用本身零分配。
    /// 自动估算不覆盖索引容器、Arc 管理开销、任意内部堆数据或外部 Arc 持有量，不执行字节准入。
    pub(crate) fn insert_with_additional_heap_bytes(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        value: V,
        additional_heap_bytes: usize,
    ) -> &V {
        let text = text.into();
        let estimated_bytes = std::mem::size_of::<TextMeasureCacheEntry<K, V>>()
            .saturating_add(text.len())
            .saturating_add(additional_heap_bytes);
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.text.as_ref() == text.as_ref());
        self.record_lookup(lookup.candidate_count);
        let slot = lookup.slot.filter(|slot| self.index.entry(*slot).is_some());
        let replaced_bytes = slot
            .and_then(|slot| self.index.entry(slot))
            .map(|entry| entry.estimated_bytes)
            .unwrap_or(0);
        if slot.is_none() {
            self.trim_before_insert();
        }
        self.estimated_bytes = self
            .estimated_bytes
            .saturating_sub(replaced_bytes)
            .saturating_add(estimated_bytes);
        self.refresh_report_bytes();
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            value,
            true,
            |entry, value| {
                entry.value = value;
                entry.estimated_bytes = estimated_bytes;
            },
            |value| TextMeasureCacheEntry {
                key,
                text,
                value,
                estimated_bytes,
            },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        } else {
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
        }
        &entry.value
    }

    /// 查询测量值，仅在未命中时调用一次构造闭包并保存结果。
    ///
    /// # 参数与返回
    /// key 与 text 共同确定身份；measure 负责产生可复用的 V，本层不认证结果状态。
    /// 返回值借用缓存，布尔值仅在新建条目时为 true；命中保留已有值且不调用闭包。
    ///
    /// # 顺序与计量
    /// 未命中先按容量淘汰旧条目，再调用闭包；原文只在新建时转成 Arc。
    /// 本入口只估算条目本体与文本长度，不能补计闭包结果的额外堆数据。
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        text: &str,
        measure: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let slot = self
            .find_slot(&key, text)
            .filter(|slot| self.index.entry(*slot).is_some());
        if slot.is_none() {
            self.trim_before_insert();
        }
        let estimated_bytes =
            std::mem::size_of::<TextMeasureCacheEntry<K, V>>().saturating_add(text.len());
        if slot.is_none() {
            self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
            self.refresh_report_bytes();
        }
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            (),
            true,
            |_, ()| {},
            |()| TextMeasureCacheEntry {
                key,
                text: Arc::from(text),
                value: measure(),
                estimated_bytes,
            },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        }
        (&entry.value, inserted)
    }

    // 先完整匹配再 touch；只有最终查找失败才记录碰撞未命中，前面的失败候选不污染成功读取。
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
        let Some(slot) = lookup.slot else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            return None;
        };

        self.index.touch(slot);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(slot)
    }

    // 为即将出现的新条目留一个位置，因此容量相等时也要先淘汰。
    fn trim_before_insert(&mut self) {
        let mut evicted = 0_u64;
        let mut eviction_scans = 0_u64;
        let mut entry_moves = 0_u64;
        while self.index.len() >= self.capacity {
            let Some((entry, work)) = self.index.pop_oldest_with_work() else {
                break;
            };
            self.estimated_bytes = self.estimated_bytes.saturating_sub(entry.estimated_bytes);
            evicted = evicted.saturating_add(1);
            eviction_scans = eviction_scans.saturating_add(work.scan_count as u64);
            entry_moves = entry_moves.saturating_add(work.entry_move_count as u64);
        }
        self.record_evictions(evicted, eviction_scans, entry_moves);
    }

    // 帧尾没有待插入条目，恰好等于容量即可保留。
    fn trim_to_capacity(&mut self) {
        let mut evicted = 0_u64;
        let mut eviction_scans = 0_u64;
        let mut entry_moves = 0_u64;
        while self.index.len() > self.capacity {
            let Some((entry, work)) = self.index.pop_oldest_with_work() else {
                break;
            };
            self.estimated_bytes = self.estimated_bytes.saturating_sub(entry.estimated_bytes);
            evicted = evicted.saturating_add(1);
            eviction_scans = eviction_scans.saturating_add(work.scan_count as u64);
            entry_moves = entry_moves.saturating_add(work.entry_move_count as u64);
        }
        self.record_evictions(evicted, eviction_scans, entry_moves);
    }

    fn record_lookup(&mut self, candidate_count: usize) {
        self.frame_report.lookup_candidate_count = self
            .frame_report
            .lookup_candidate_count
            .saturating_add(candidate_count as u64);
    }

    // 工作量始终累计；只有实际移除了条目才记一次裁剪，清空不经过此路径。
    fn record_evictions(&mut self, evicted: u64, eviction_scans: u64, entry_moves: u64) {
        self.frame_report.eviction_scan_count = self
            .frame_report
            .eviction_scan_count
            .saturating_add(eviction_scans);
        self.frame_report.entry_move_count = self
            .frame_report
            .entry_move_count
            .saturating_add(entry_moves);
        if evicted > 0 {
            self.frame_report.evicted_count =
                self.frame_report.evicted_count.saturating_add(evicted);
            self.frame_report.trim_count = self.frame_report.trim_count.saturating_add(1);
        }
        self.refresh_report_size();
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.index.len();
        self.frame_report.estimated_bytes = self.estimated_bytes;
    }

    fn refresh_report_bytes(&mut self) {
        self.frame_report.estimated_bytes = self.estimated_bytes;
        self.frame_report.peak_estimated_bytes = self
            .frame_report
            .peak_estimated_bytes
            .max(self.estimated_bytes);
    }
}
