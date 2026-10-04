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

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829av_runtime322_single_lookup_text_cache_lru_detach_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
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

fn populated_cache() -> (IndexedTextCache<u64, Entry>, Vec<TextCacheSlot>) {
    let mut cache = IndexedTextCache::new();
    let slots = (0..ENTRY_COUNT as u64)
        .map(|key| cache.insert(Entry { key }))
        .collect();
    (cache, slots)
}

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

fn legacy_touch(cache: &mut IndexedTextCache<u64, Entry>, slot: TextCacheSlot) {
    if !cache.entries.contains_key(&slot) {
        return;
    }
    if cache.lru_links.contains_key(&slot) {
        legacy_detach_lru(cache, slot);
    }
    cache.attach_most_recent(slot);
}

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

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
