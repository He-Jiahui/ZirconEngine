//! 触碰和摘链顺序的回归夹具，以及显式忽略的独立缓存对照计时。
//! 计时输出的探针数是固定模型值，性能结论依赖显式运行所得的样本。
use std::hint::black_box;
use std::time::Instant;

use super::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

const ENTRY_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 31;
const TOUCH_PASSES: usize = 16;

#[derive(Clone, Debug, PartialEq)]
struct Entry {
    key: u64,
}

impl IndexedTextCacheEntry<u64> for Entry {
    fn cache_key(&self) -> &u64 {
        &self.key
    }
}

#[test]
fn optimization_batch_20260829av_runtime322_middle_touch_preserves_lru_order() {
    let mut cache = IndexedTextCache::new();
    cache.insert(Entry { key: 1 });
    let middle = cache.insert(Entry { key: 2 });
    cache.insert(Entry { key: 3 });

    cache.touch(middle);

    assert_eq!(cache.pop_oldest().map(|entry| entry.key), Some(1));
    assert_eq!(cache.pop_oldest().map(|entry| entry.key), Some(3));
    assert_eq!(cache.pop_oldest().map(|entry| entry.key), Some(2));
}

// 只覆盖陈旧槽、未跟踪槽和指定后邻居缺失的人工状态。
#[test]
fn optimization_batch_20260829av_runtime322_touch_attaches_untracked_and_ignores_stale_slot() {
    let mut cache = IndexedTextCache::new();
    let slot = cache.insert_untracked(Entry { key: 7 });

    cache.touch(u64::MAX);
    cache.touch(slot);

    assert_eq!(cache.pop_oldest().map(|entry| entry.key), Some(7));
    assert!(cache.pop_oldest().is_none());

    let first = cache.insert(Entry { key: 11 });
    let missing_next = cache.insert(Entry { key: 12 });
    cache.lru_links.remove(&missing_next);
    cache.touch(first);

    assert_eq!(cache.pop_oldest().map(|entry| entry.key), Some(11));
    cache.touch(missing_next);
    assert_eq!(cache.pop_oldest().map(|entry| entry.key), Some(12));
}

/// 在受管环境显式启用的计时对照；默认忽略，普通回归只检查索引结果。
/// 每对样本使用内容相同的独立缓存，并交替两种实现的计时次序。
/// 打印的探针常量不是运行时采样，最终断言比较本次样本的 p95。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829av_runtime322_single_lookup_text_cache_lru_detach_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        // 建表与克隆发生在各自计时函数外，不计入样本。
        let (base, slots) = populated_cache();
        if pair % 2 == 0 {
            legacy_samples.push(measure(base.clone(), &slots, false));
            optimized_samples.push(measure(base, &slots, true));
        } else {
            optimized_samples.push(measure(base.clone(), &slots, true));
            legacy_samples.push(measure(base, &slots, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME322_SINGLE_LOOKUP_TEXT_CACHE_LRU_DETACH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
entries={ENTRY_COUNT} touch_passes={TOUCH_PASSES} legacy_hash_probes_per_middle_touch=9 \
optimized_hash_probes_per_middle_touch=6 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

/// 为一对样本建立完整活槽列表，缓存随后分别克隆或移交给两次计时。
fn populated_cache() -> (IndexedTextCache<u64, Entry>, Vec<TextCacheSlot>) {
    let mut cache = IndexedTextCache::new();
    let slots = (0..ENTRY_COUNT as u64)
        .map(|key| cache.insert(Entry { key }))
        .collect();
    (cache, slots)
}

/// 消费独立缓存并返回耗时纳秒（最小一）；调用者保证槽列表非空且来自该缓存。
/// `optimized` 仅选择摘链实现，两条路径均包含循环、槽选择及校验和的成本。
/// 每份缓存完成 16 × 4096 次触碰，`black_box` 保留输入与结果的可观察使用。
/// 样本间不共享最近性状态。
fn measure(
    mut cache: IndexedTextCache<u64, Entry>,
    slots: &[TextCacheSlot],
    optimized: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u64;
    for pass in 0..TOUCH_PASSES {
        for offset in 0..slots.len() {
            let slot = slots[(offset * 2_053 + pass) % slots.len()];
            if optimized {
                cache.touch(black_box(slot));
            } else {
                legacy_touch(&mut cache, black_box(slot));
            }
            checksum ^= cache.lru_tail.unwrap_or_default();
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

/// 仅供基准对照的旧版探针顺序，不是生产回退入口。
fn legacy_touch(cache: &mut IndexedTextCache<u64, Entry>, slot: TextCacheSlot) {
    if !cache.entries.contains_key(&slot) {
        return;
    }
    if cache.lru_links.contains_key(&slot) {
        legacy_detach_lru(cache, slot);
    }
    cache.attach_most_recent(slot);
}

/// 旧实现先分别筛选邻居再改链，仅由本文件旧触碰路径调用。
fn legacy_detach_lru(cache: &mut IndexedTextCache<u64, Entry>, slot: TextCacheSlot) {
    let Some(links) = cache.lru_links.remove(&slot) else {
        return;
    };
    let previous = links
        .previous
        .filter(|candidate| cache.lru_links.contains_key(candidate));
    let next = links
        .next
        .filter(|candidate| cache.lru_links.contains_key(candidate));
    if let Some(previous) = previous {
        if let Some(previous_links) = cache.lru_links.get_mut(&previous) {
            previous_links.next = next;
        }
    } else {
        cache.lru_head = next;
    }
    if let Some(next) = next {
        if let Some(next_links) = cache.lru_links.get_mut(&next) {
            next_links.previous = previous;
        }
    } else {
        cache.lru_tail = previous;
    }
}

/// 对非空样本复制排序，按向上取整的百分位秩返回原单位值。
/// 本文件只传入 31 项以及 50、95，不承担空输入或任意百分位校验。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

/// 保留采样次序输出逗号分隔值，便于回执核对原始样本。
fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
