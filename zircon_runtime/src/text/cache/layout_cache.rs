//! UI 布局的跨帧驻留缓存；请求键、完整文本和已认证的宽度区间共同决定可否复用。
//! UI owner 在帧内去重未命中后查询本层；可复用结果与宽度承诺由调用方提供。
//! 条目跨帧驻留，容量按条目限制；字节报告仅作估算遥测，借用返回值依赖缓存仍被借用。
//! 泛型层不验证布局状态、几何或字体代次；这些条件应在构造键及写入前由调用方处理。
use std::{hash::Hash, sync::Arc};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

pub(crate) const DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY: usize = 2048;

/// 调用方对布局不随宽度变化的区间作出的证明；Range 的上界不包含在内，未经认证时使用 Exact。
///
/// # 约束
/// 构造函数只保存宽度位模式，不证明布局在区间内不变，也不拒绝异常或倒置端点。
/// Exact 归一正负零后比较位模式，不要求有限；Range 才要求端点与查询值有限并使用半开区间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextLayoutWidthValidity {
    Exact(u32),
    Range {
        min_width_bits: u32,
        max_width_bits: u32,
    },
}

impl TextLayoutWidthValidity {
    /// 将一个宽度编码为精确匹配条件；仅归一正负零，保留其他位模式。
    pub(crate) fn exact(width: f32) -> Self {
        Self::Exact(normalized_width_bits(width))
    }

    /// 保存调用方承诺的 [min_width, max_width)；本构造不验证端点或承诺。
    pub(crate) fn range(min_width: f32, max_width: f32) -> Self {
        Self::Range {
            min_width_bits: normalized_width_bits(min_width),
            max_width_bits: normalized_width_bits(max_width),
        }
    }

    /// 判断查询宽度是否落入已保存的有效性条件，不重新认证布局。
    ///
    /// Exact 可匹配相同的非有限位模式；Range 仅在三个值均有限且 min <= width < max 时返回 true。
    pub(crate) fn contains(self, width: f32) -> bool {
        match self {
            Self::Exact(bits) => normalized_width_bits(width) == bits,
            Self::Range {
                min_width_bits,
                max_width_bits,
            } => {
                let min_width = f32::from_bits(min_width_bits);
                let max_width = f32::from_bits(max_width_bits);
                min_width.is_finite()
                    && max_width.is_finite()
                    && width.is_finite()
                    && min_width <= width
                    && width < max_width
            }
        }
    }
}

/// 当前帧的逻辑操作计数和驻留估算快照。
///
/// # 计量口径
/// 字节估算仅包含条目本体、文本长度和显式补计值，不能作为进程内存或分配器用量上限。
/// 碰撞与宽度失败各自只在最终未命中时累计，同一次失败可同时增加两项。
/// 插入只计候选访问及插入/更新；峰值从帧开始的驻留估算起算，清空不抹去已出现的峰值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextLayoutCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) peak_estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) width_miss_count: u64,
    pub(crate) lookup_candidate_count: u64,
    pub(crate) eviction_scan_count: u64,
    pub(crate) entry_move_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

/// 索引持有的布局记录；原文与有效性共同区分同键候选，单项估算供替换和淘汰扣除。
#[derive(Clone, Debug, PartialEq)]
struct TextLayoutCacheEntry<K, V> {
    key: K,
    text: Arc<str>,
    width_validity: TextLayoutWidthValidity,
    value: V,
    estimated_bytes: usize,
}

impl<K, V> IndexedTextCacheEntry<K> for TextLayoutCacheEntry<K, V> {
    fn cache_key(&self) -> &K {
        &self.key
    }
}

/// UI 布局 owner 在帧内去重未命中后查询本缓存，只有已完成的布局才可写入。
///
/// # 使用与生命周期
/// 调用方将影响结果的状态编码进 K；泛型 V 不携带本层能检查的完成状态，写入准入由调用方负责。
/// 读取返回缓存内借用；共享原文只有在显式克隆 Arc 后才可独立于该借用继续持有。
///
/// # 容量与宽度
/// 容量至少为 1，限制条目数而非估算字节；读取按已有条目的有效性判断，不接受新的复用证明。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextLayoutCache<K: Eq + Hash, V> {
    index: IndexedTextCache<K, TextLayoutCacheEntry<K, V>>,
    capacity: usize,
    estimated_bytes: usize,
    frame_report: TextLayoutCacheReport,
}

impl<K, V> Default for TextLayoutCache<K, V>
where
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> TextLayoutCache<K, V>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn new() -> Self {
        Self::with_capacity(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY)
    }

    /// 创建空缓存，设置逻辑条目上限；不按该数预留索引内存。
    ///
    /// capacity 为驻留条目数，0 按 1 处理；返回尚无条目、计数为零的 owner。
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut cache = Self {
            index: IndexedTextCache::new(),
            capacity,
            estimated_bytes: 0,
            frame_report: TextLayoutCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache
    }

    /// 开始一帧的统计，保留驻留条目及 LRU 次序。
    ///
    /// frame_index 只作报告标签；帧计数归零，峰值从当前驻留估算起算，不验证代次单调性。
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.frame_report = TextLayoutCacheReport {
            frame_index,
            capacity: self.capacity,
            entry_count: self.index.len(),
            estimated_bytes: self.estimated_bytes,
            peak_estimated_bytes: self.estimated_bytes,
            ..TextLayoutCacheReport::default()
        };
    }

    /// 帧尾只裁掉超过容量的条目，不移除仅因本帧未引用的值。
    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_capacity();
    }

    /// 释放全部索引条目，将移除数与清空次数累入报告，并将当前驻留估算归零。
    ///
    /// 保留本帧峰值和其他历史计数，不增加容量裁剪工作量。
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

    /// 复制报告并刷新当前条目数、驻留估算；不改变 LRU 或计数。
    pub(crate) fn report(&self) -> TextLayoutCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    /// 只检查同键、完整原文及整个宽度有效性值均相等的条目。
    ///
    /// 这是条目身份判断，不是对某个查询宽度的包含判断；返回是否存在，不 touch 或累计读取计数。
    pub(crate) fn contains_exact(
        &self,
        key: &K,
        text: &str,
        width_validity: TextLayoutWidthValidity,
    ) -> bool {
        self.index
            .find_slot(key, |entry| {
                entry.text.as_ref() == text && entry.width_validity == width_validity
            })
            .slot
            .is_some()
    }

    /// 按保存的宽度条件查询并借用值；读取副作用和生命周期见 Self::get_with_stored_text。
    pub(crate) fn get(&mut self, key: &K, text: &str, wrap_width: f32) -> Option<&V> {
        self.get_with_stored_text(key, text, wrap_width)
            .map(|(_, value)| value)
    }

    /// 查询可复用的布局，并同时借出驻留原文与值。
    ///
    /// # 参数与返回
    /// key 定位候选，text 排除完整原文不同的别名；wrap_width 交给已有条目的有效性条件判断。
    /// 失败返回 None；成功会 touch LRU 并累计读取命中，返回 &Arc 与 &V，均受缓存借用期约束。
    /// 需要跨后续缓存修改持有原文时，由调用方克隆 Arc；本函数不转移缓存内 Arc 的所有权。
    pub(crate) fn get_with_stored_text(
        &mut self,
        key: &K,
        text: &str,
        wrap_width: f32,
    ) -> Option<(&Arc<str>, &V)> {
        let slot = self.find_slot(key, text, wrap_width)?;
        self.index
            .entry(slot)
            .map(|entry| (&entry.text, &entry.value))
    }

    /// 接收键、共享原文、宽度承诺和值，返回更新或插入后值的缓存借用。
    ///
    /// 本入口额外堆估算为零；完整身份与计量规则见 Self::insert_with_additional_heap_bytes。
    pub(crate) fn insert(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        width_validity: TextLayoutWidthValidity,
        value: V,
    ) -> &V {
        self.insert_with_additional_heap_bytes(key, text, width_validity, value, 0)
    }

    /// 调用方补计 K 和 V 的堆数据；缓存只自动计入条目本体与原文，字节报告用于预算观察。
    ///
    /// # 参数与返回
    /// key、text、width_validity 与 value 转交缓存；额外堆估算由 additional_heap_bytes 显式提供。
    /// 只有同键、完整原文和整个有效性值相等才更新，保留原键、Arc 与有效性；返回新值的借用。
    ///
    /// # 约束与顺序
    /// 新条目先腾出容量位置；text 的转换先于查找，不承诺本次调用零分配。
    /// 调用方负责有效性承诺；自动估算不包括索引容器、Arc 管理开销或未补计的堆数据，也不执行字节准入。
    pub(crate) fn insert_with_additional_heap_bytes(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        width_validity: TextLayoutWidthValidity,
        value: V,
        additional_heap_bytes: usize,
    ) -> &V {
        let text = text.into();
        let estimated_bytes = std::mem::size_of::<TextLayoutCacheEntry<K, V>>()
            .saturating_add(text.len())
            .saturating_add(additional_heap_bytes);
        let lookup = self.index.find_slot(&key, |entry| {
            entry.text.as_ref() == text.as_ref() && entry.width_validity == width_validity
        });
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
            |value| TextLayoutCacheEntry {
                key,
                text,
                width_validity,
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

    /// 查询布局，仅在没有可复用条目时调用一次构造闭包。
    ///
    /// # 参数与返回
    /// key 与 text 共同定位内容；wrap_width 用于查询已有有效性，width_validity 只供新条目保存。
    /// layout 负责产生符合该承诺的 V；命中保留已有有效性和值，忽略新传入的承诺且不调用闭包。
    /// 返回值借用缓存，布尔值仅在新建条目时为 true。
    ///
    /// # 顺序与计量
    /// 未命中先按容量淘汰旧条目，再调用闭包；仅新建时分配共享原文。
    /// 本入口只估算条目本体与文本长度，不补计闭包结果的额外堆数据。
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        text: &str,
        width_validity: TextLayoutWidthValidity,
        wrap_width: f32,
        layout: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let slot = self
            .find_slot(&key, text, wrap_width)
            .filter(|slot| self.index.entry(*slot).is_some());
        if slot.is_none() {
            self.trim_before_insert();
        }
        let estimated_bytes =
            std::mem::size_of::<TextLayoutCacheEntry<K, V>>().saturating_add(text.len());
        if slot.is_none() {
            self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
            self.refresh_report_bytes();
        }
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            (),
            true,
            |_, ()| {},
            |()| TextLayoutCacheEntry {
                key,
                text: Arc::from(text),
                width_validity,
                value: layout(),
                estimated_bytes,
            },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        }
        (&entry.value, inserted)
    }

    // 碰撞和宽度失败先收集为本次候选现象；最终命中只记 hit，最终失败才分别累计两个原因。
    fn find_slot(&mut self, key: &K, text: &str, wrap_width: f32) -> Option<TextCacheSlot> {
        let mut collision_seen = false;
        let mut width_miss_seen = false;
        let lookup = self.index.find_slot(key, |entry| {
            if entry.text.as_ref() != text {
                collision_seen = true;
                return false;
            }
            if !entry.width_validity.contains(wrap_width) {
                width_miss_seen = true;
                return false;
            }
            true
        });
        self.record_lookup(lookup.candidate_count);
        let Some(slot) = lookup.slot else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            if width_miss_seen {
                self.frame_report.width_miss_count =
                    self.frame_report.width_miss_count.saturating_add(1);
            }
            return None;
        };

        self.index.touch(slot);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(slot)
    }

    // 为即将出现的新条目留位，容量相等时也先淘汰。
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

    // 帧尾不需要预留位置，因此保留恰好达到容量的集合。
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

    // 只有实际移除条目才记一次裁剪；扫描与移动量来自索引返回的逻辑工作计数。
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

// 仅消除正负零的身份差异；其余位模式原样保留，由有效性分支决定能否接受。
fn normalized_width_bits(width: f32) -> u32 {
    if width == 0.0 {
        0.0_f32.to_bits()
    } else {
        width.to_bits()
    }
}
