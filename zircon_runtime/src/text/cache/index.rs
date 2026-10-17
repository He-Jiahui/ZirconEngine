//! 测量、布局与帧内去重共用的索引：稳定槽位承载碰撞桶，只有持久缓存把槽位接入淘汰链。
//!
//! # 边界与生命周期
//! 各缓存 owner 负责预算、完整文本与布局条件；本层只维护条目、碰撞桶和可选 LRU 链。
//! 槽号仅在所属索引的活条目中标识身份，不表示物理地址或跨实例、永久唯一身份。
//! 参见同级帧内去重、测量、布局及塑形缓存；直属测试区分索引不变量与显式忽略的计时夹具。
use std::{collections::HashMap, hash::Hash};

/// 所属索引中活条目的逻辑句柄；移除或清空后不得把旧值当成仍有效的身份。
/// 计数回绕可能复用已不存活的值；不同实例或克隆之间也不能只凭槽号比较条目。
pub(super) type TextCacheSlot = u64;

/// 首个匹配槽及截至该处的访问量；候选顺序由桶或调用方提供。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TextCacheLookup {
    /// 首个通过调用方谓词的活槽；未命中时为 None。
    pub(super) slot: Option<TextCacheSlot>,
    /// 实际调用匹配谓词的活条目数，跳过缺失槽且命中即停止。
    pub(super) candidate_count: usize,
}

/// 本索引报告的一次淘汰工作量，不计量底层容器的物理移动或哈希探针。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TextCacheEvictionWork {
    /// 从链头直接选出条目时报告一个候选。
    pub(super) scan_count: usize,
    /// 不重排其他条目来淘汰；桶内槽号交换不计入此字段。
    pub(super) entry_move_count: usize,
}

/// 驻留期间 cache_key 必须保持不变；可变访问及更新回调只能修改结果或统计，不能让桶与条目身份脱节。
pub(crate) trait IndexedTextCacheEntry<K> {
    /// 借用用于分桶的身份键；条目存活期间须与登记的桶键一致。
    fn cache_key(&self) -> &K;
}

/// 已跟踪槽的双向邻接；两端的 None 分别表示链头和链尾。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TextCacheLruLinks {
    previous: Option<TextCacheSlot>,
    next: Option<TextCacheSlot>,
}

// Slots keep entry addresses independent from collision-bucket maintenance.
// The linked index owns recency, making touch and eviction constant time.
/// 统一拥有缓存条目，并以逻辑槽号连接碰撞桶和可选的淘汰链。
///
/// # 所有权与约束
/// owner 决定完整身份、容量及附加索引；这里不按键去重，也不验证完整文本。
/// 通过可变借用或回调更新条目时必须保持键不变；未跟踪条目仍参与查找。
///
/// # 参考
/// 同级的帧内去重使用未跟踪条目，测量、布局和塑形缓存用链头选择淘汰对象。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IndexedTextCache<K: Eq + Hash, E> {
    /// 活条目的拥有者；其他表只保存槽号或键的副本。
    entries: HashMap<TextCacheSlot, E>,
    /// 按请求键分桶；桶内顺序可能因交换移除而改变。
    buckets: HashMap<K, Vec<TextCacheSlot>>,
    /// 槽在所属碰撞桶中的位置，用于直接定位移除项。
    bucket_positions: HashMap<TextCacheSlot, usize>,
    /// 仅收录参与淘汰的槽，未跟踪条目可以只存在于条目表和桶中。
    lru_links: HashMap<TextCacheSlot, TextCacheLruLinks>,
    lru_head: Option<TextCacheSlot>,
    lru_tail: Option<TextCacheSlot>,
    /// 下次搜索起点；清空保留计数，回绕时只避开当前活槽。
    next_slot: TextCacheSlot,
}

impl<K, E> IndexedTextCache<K, E>
where
    K: Clone + Eq + Hash,
    E: IndexedTextCacheEntry<K>,
{
    pub(super) fn new() -> Self {
        Self {
            entries: HashMap::new(),
            buckets: HashMap::new(),
            bucket_positions: HashMap::new(),
            lru_links: HashMap::new(),
            lru_head: None,
            lru_tail: None,
            next_slot: 1,
        }
    }

    /// 清空条目、桶及淘汰链，使旧句柄失效；保留槽号分配器的搜索起点。
    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.buckets.clear();
        self.bucket_positions.clear();
        self.lru_links.clear();
        self.lru_head = None;
        self.lru_tail = None;
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 借用当前活条目；不校验 owner 的完整身份，也不改变最近性。
    pub(super) fn entry(&self, slot: TextCacheSlot) -> Option<&E> {
        self.entries.get(&slot)
    }

    /// 独占借用活条目；调用方只能更新不改变分桶身份的内容。
    pub(super) fn entry_mut(&mut self, slot: TextCacheSlot) -> Option<&mut E> {
        self.entries.get_mut(&slot)
    }

    /// 按键选桶，再由调用方谓词验证完整身份；查询本身不更新 LRU。
    ///
    /// # 参数与返回
    /// `matches` 负责完整文本及布局条件；返回首个命中槽和已访问的活条目数。
    /// 无桶时返回默认空结果；统计不包含缺失槽或 HashMap 探针。
    pub(super) fn find_slot(&self, key: &K, matches: impl FnMut(&E) -> bool) -> TextCacheLookup {
        let Some(candidates) = self.buckets.get(key) else {
            return TextCacheLookup::default();
        };

        self.find_in_slots(candidates, matches)
    }

    /// 顺序检查调用方给出的候选，跳过缺失条目，首次匹配立即返回。
    /// 计数只覆盖真正调用谓词的条目；本层不验证候选是否属于同一键。
    pub(super) fn find_in_slots(
        &self,
        candidates: &[TextCacheSlot],
        mut matches: impl FnMut(&E) -> bool,
    ) -> TextCacheLookup {
        let mut candidate_count = 0;
        for &slot in candidates {
            let Some(entry) = self.entries.get(&slot) else {
                continue;
            };
            candidate_count += 1;
            if matches(entry) {
                return TextCacheLookup {
                    slot: Some(slot),
                    candidate_count,
                };
            }
        }

        TextCacheLookup {
            slot: None,
            candidate_count,
        }
    }

    /// 接管新条目并接到最近端，返回逻辑槽号；完整身份去重由 owner 先行完成。
    pub(super) fn insert(&mut self, entry: E) -> TextCacheSlot {
        let (slot, _) = self.insert_inner(entry, true);
        slot
    }

    /// 区分只参与查找的条目与持久驻留条目；若要参加淘汰，owner 须显式 touch。
    /// 返回槽仍进入查找索引；不跟踪不等同于条目不存在。
    pub(super) fn insert_untracked(&mut self, entry: E) -> TextCacheSlot {
        let (slot, _) = self.insert_inner(entry, false);
        slot
    }

    /// 调用方先按完整文本和必要的布局条件筛出槽位；这里只维护该槽位的桶与淘汰索引。
    ///
    /// # 参数与返回
    /// `update_slot` 必须已由 owner 验证；活槽会直接更新，不再次核对其键。
    /// `track_lru=true` 先更新最近性再调用 `update`；false 不会把既有已跟踪槽摘链。
    /// `input` 只交给实际选中的一个回调；`update` 必须保持条目的键不变。
    /// 返回槽、独占借用及是否新建；缺失槽通过分配器重新登记，不接受外来槽作为指定身份。
    ///
    /// # 生命周期
    /// 回调须保持身份约束；本方法不承诺修复任意损坏的桶或链。
    pub(super) fn update_or_insert_with<T>(
        &mut self,
        update_slot: Option<TextCacheSlot>,
        input: T,
        track_lru: bool,
        update: impl FnOnce(&mut E, T),
        make_entry: impl FnOnce(T) -> E,
    ) -> (TextCacheSlot, &mut E, bool) {
        let Some(slot) = update_slot else {
            let (slot, entry) = self.insert_inner(make_entry(input), track_lru);
            return (slot, entry, true);
        };

        if !self.entries.contains_key(&slot) {
            // Do not reuse a stale slot: a malformed bucket may still reference
            // it. A fresh slot keeps that bucket fail-closed and records the new
            // entry through every authoritative index.
            let (slot, entry) = self.insert_inner(make_entry(input), track_lru);
            return (slot, entry, true);
        }

        if track_lru {
            self.touch(slot);
        }

        match self.entries.entry(slot) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                update(occupied.get_mut(), input);
                (slot, occupied.into_mut(), false)
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                // The caller supplied a stale slot. Register the recovered entry
                // before returning it so lookup and LRU ownership stay coherent.
                let entry = make_entry(input);
                let key = entry.cache_key().clone();
                let candidates = self.buckets.entry(key).or_default();
                let candidate_index = candidates.len();
                candidates.push(slot);
                self.bucket_positions.insert(slot, candidate_index);
                if track_lru {
                    Self::attach_most_recent_parts(
                        &mut self.lru_links,
                        &mut self.lru_head,
                        &mut self.lru_tail,
                        slot,
                    );
                }
                (slot, vacant.insert(entry), true)
            }
        }
    }

    /// 选取空闲逻辑槽，登记桶位置及可选链，再返回条目的独占借用。
    fn insert_inner(&mut self, entry: E, track_lru: bool) -> (TextCacheSlot, &mut E) {
        let slot = self.next_slot();
        let key = entry.cache_key().clone();
        let candidates = self.buckets.entry(key).or_default();
        let candidate_index = candidates.len();
        candidates.push(slot);
        self.bucket_positions.insert(slot, candidate_index);
        if track_lru {
            self.attach_most_recent(slot);
        }
        let entry = match self.entries.entry(slot) {
            std::collections::hash_map::Entry::Vacant(vacant) => vacant.insert(entry),
            std::collections::hash_map::Entry::Occupied(occupied) => occupied.into_mut(),
        };
        (slot, entry)
    }

    /// 将活槽置于最近端，未跟踪槽也可由此入链；不存在的槽被忽略。
    pub(super) fn touch(&mut self, slot: TextCacheSlot) {
        if !self.entries.contains_key(&slot) {
            return;
        }
        self.detach_lru(slot);
        self.attach_most_recent(slot);
    }

    /// 移出已跟踪链头并交出条目所有权；未跟踪条目不参与选择。
    pub(super) fn pop_oldest(&mut self) -> Option<E> {
        self.pop_oldest_with_work().map(|(entry, _)| entry)
    }

    /// 同时交回移除前槽号，供 owner 清理旁路索引；该槽此后不再指向原条目。
    pub(super) fn pop_oldest_with_slot(&mut self) -> Option<(TextCacheSlot, E)> {
        self.pop_oldest_with_slot_and_work()
            .map(|(slot, entry, _)| (slot, entry))
    }

    /// 淘汰条目并附带逻辑工作计量；链空时返回 None。
    pub(super) fn pop_oldest_with_work(&mut self) -> Option<(E, TextCacheEvictionWork)> {
        self.pop_oldest_with_slot_and_work()
            .map(|(_, entry, work)| (entry, work))
    }

    /// 返回旧槽号、拥有的最旧条目及工作计量，由 owner 同步清理自身附加索引。
    /// 一次候选扫描与零条目搬移是报告口径，不证明容器物理零移动或哈希最坏常数成本。
    pub(super) fn pop_oldest_with_slot_and_work(
        &mut self,
    ) -> Option<(TextCacheSlot, E, TextCacheEvictionWork)> {
        let slot = self.lru_head?;
        self.remove(slot).map(|entry| {
            (
                slot,
                entry,
                TextCacheEvictionWork {
                    // The linked LRU head identifies the victim directly. No resident-entry
                    // search or stable-entry relocation is hidden behind the cache report.
                    scan_count: 1,
                    entry_move_count: 0,
                },
            )
        })
    }

    /// 按槽移除条目、链关系及桶位置；返回的条目由调用方接管。
    /// 依赖条目的键在驻留期间不变；本方法不遍历其他桶来修复错误身份。
    pub(super) fn remove(&mut self, slot: TextCacheSlot) -> Option<E> {
        let entry = self.entries.remove(&slot)?;
        self.detach_lru(slot);

        let key = entry.cache_key();
        let remove_bucket = if let Some(candidates) = self.buckets.get_mut(key) {
            if let Some(index) = self.bucket_positions.remove(&slot) {
                // 交换的是桶内槽号；只需把末项的新位置写回反向索引。
                let removed = candidates.swap_remove(index);
                debug_assert_eq!(removed, slot);
                if let Some(&moved_slot) = candidates.get(index) {
                    self.bucket_positions.insert(moved_slot, index);
                }
            }
            candidates.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.buckets.remove(key);
        }
        Some(entry)
    }

    /// 从计数器起点找当前未占用的槽；回绕不提供世代隔离或永久唯一性。
    fn next_slot(&mut self) -> TextCacheSlot {
        loop {
            let slot = self.next_slot;
            self.next_slot = self.next_slot.checked_add(1).unwrap_or(1);
            if !self.entries.contains_key(&slot) {
                return slot;
            }
        }
    }

    /// 将已脱链或新分配的槽接到最近端；调用方负责避免重复挂接。
    fn attach_most_recent(&mut self, slot: TextCacheSlot) {
        Self::attach_most_recent_parts(
            &mut self.lru_links,
            &mut self.lru_head,
            &mut self.lru_tail,
            slot,
        );
    }

    /// 分离借用链表字段，供条目表被独占借用的更新分支重用。
    fn attach_most_recent_parts(
        lru_links: &mut HashMap<TextCacheSlot, TextCacheLruLinks>,
        lru_head: &mut Option<TextCacheSlot>,
        lru_tail: &mut Option<TextCacheSlot>,
        slot: TextCacheSlot,
    ) {
        let previous = *lru_tail;
        lru_links.insert(
            slot,
            TextCacheLruLinks {
                previous,
                next: None,
            },
        );
        if let Some(previous) = previous {
            if let Some(previous_links) = lru_links.get_mut(&previous) {
                previous_links.next = Some(slot);
            } else {
                *lru_head = Some(slot);
            }
        } else {
            *lru_head = Some(slot);
        }
        *lru_tail = Some(slot);
    }

    /// 只摘除指定槽及其直接邻接关系；缺失自身链接时保持其余状态。
    /// 对缺失邻链接作局部截断，不遍历、验证或修复任意损坏链。
    fn detach_lru(&mut self, slot: TextCacheSlot) {
        let Some(links) = self.lru_links.remove(&slot) else {
            return;
        };
        let mut previous = links.previous;
        if let Some(candidate) = previous {
            if let Some(previous_links) = self.lru_links.get_mut(&candidate) {
                previous_links.next = links.next;
            } else {
                previous = None;
                self.lru_head = links.next;
            }
        } else {
            self.lru_head = links.next;
        }

        let next = if let Some(candidate) = links.next {
            if let Some(next_links) = self.lru_links.get_mut(&candidate) {
                next_links.previous = previous;
                Some(candidate)
            } else {
                if let Some(previous) = previous {
                    if let Some(previous_links) = self.lru_links.get_mut(&previous) {
                        previous_links.next = None;
                    }
                } else {
                    self.lru_head = None;
                }
                None
            }
        } else {
            None
        };
        if next.is_none() {
            self.lru_tail = previous;
        }
    }
}

#[cfg(test)]
#[path = "tests/index.rs"]
mod tests;

#[cfg(test)]
#[path = "index/tests/detach_single_lookup_tests.rs"]
mod detach_single_lookup_tests;
