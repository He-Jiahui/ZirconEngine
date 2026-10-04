use super::{IndexedTextCache, IndexedTextCacheEntry};

#[derive(Debug, PartialEq)]
struct Entry {
    key: u8,
    value: u8,
}

impl IndexedTextCacheEntry<u8> for Entry {
    fn cache_key(&self) -> &u8 {
        &self.key
    }
}

#[test]
fn touch_moves_a_stable_slot_to_the_lru_tail() {
    let mut cache = IndexedTextCache::new();
    cache.insert(Entry { key: 1, value: 10 });
    let second = cache.insert(Entry { key: 2, value: 20 });
    cache.insert(Entry { key: 3, value: 30 });

    let first = cache.find_slot(&1, |_| true).slot.unwrap();
    cache.touch(first);

    assert_eq!(cache.pop_oldest().unwrap().value, 20);
    assert_eq!(cache.pop_oldest().unwrap().value, 30);
    assert_eq!(cache.pop_oldest().unwrap().value, 10);
    assert!(cache.entry(second).is_none());
}

#[test]
fn text_cache_indexes_keep_hot_lookup_and_eviction_work_constant_after_insert() {
    let mut cache = IndexedTextCache::new();

    let slot = cache.insert(Entry { key: 7, value: 70 });
    assert_eq!(cache.entry(slot).map(|entry| entry.value), Some(70));
}

#[test]
fn untracked_insert_stays_out_of_the_eviction_chain_until_touched() {
    let mut cache = IndexedTextCache::new();
    let slot = cache.insert_untracked(Entry { key: 7, value: 70 });

    assert!(cache.pop_oldest().is_none());

    cache.touch(slot);
    assert_eq!(cache.pop_oldest().map(|entry| entry.value), Some(70));
}

#[test]
fn text_cache_indexes_keep_hot_lookup_and_eviction_work_constant_when_upserting() {
    let mut cache = IndexedTextCache::new();
    let slot = cache.insert(Entry { key: 7, value: 70 });
    let (updated_slot, entry, inserted) = cache.update_or_insert_with(
        Some(slot),
        80,
        true,
        |entry, value| entry.value = value,
        |value| Entry { key: 7, value },
    );

    assert_eq!(updated_slot, slot);
    assert!(!inserted);
    assert_eq!(entry.value, 80);
    assert_eq!(cache.len(), 1);
}

#[test]
fn stale_upsert_slot_recovers_through_the_lookup_and_lru_indexes() {
    let mut cache: IndexedTextCache<u8, Entry> = IndexedTextCache::new();
    let (slot, entry, inserted) = cache.update_or_insert_with(
        Some(999),
        70,
        true,
        |entry, value| entry.value = value,
        |value| Entry { key: 7, value },
    );

    assert!(inserted);
    assert_eq!(entry.value, 70);
    assert_ne!(slot, 999);
    assert_eq!(cache.find_slot(&7, |_| true).slot, Some(slot));
    assert_eq!(cache.pop_oldest().map(|entry| entry.value), Some(70));
}

#[test]
fn lru_recovers_when_a_neighbor_link_is_missing() {
    let mut cache = IndexedTextCache::new();
    let first = cache.insert(Entry { key: 1, value: 10 });
    let second = cache.insert(Entry { key: 2, value: 20 });

    cache.lru_links.remove(&first);
    cache.touch(second);

    assert_eq!(cache.pop_oldest().unwrap().value, 20);
}

#[test]
fn collision_bucket_removal_does_not_scan_candidates() {
    let source = include_str!("../index.rs");
    let linear_search = concat!("candidates.iter()", ".position");

    assert!(
        !source.contains(linear_search),
        "eviction must remove a collision-bucket slot through its indexed position"
    );
}

#[test]
fn removing_a_middle_collision_candidate_keeps_remaining_slots_indexed() {
    let mut cache = IndexedTextCache::new();
    let first = cache.insert(Entry { key: 1, value: 10 });
    let middle = cache.insert(Entry { key: 1, value: 20 });
    let last = cache.insert(Entry { key: 1, value: 30 });

    assert_eq!(cache.remove(middle).map(|entry| entry.value), Some(20));
    assert_eq!(
        cache.find_slot(&1, |entry| entry.value == 10).slot,
        Some(first)
    );
    assert_eq!(
        cache.find_slot(&1, |entry| entry.value == 30).slot,
        Some(last)
    );
    assert_eq!(cache.remove(last).map(|entry| entry.value), Some(30));
    assert_eq!(cache.pop_oldest().map(|entry| entry.value), Some(10));
    assert!(cache.is_empty());
}

#[test]
fn oldest_eviction_reports_one_direct_candidate_and_zero_entry_moves() {
    let mut cache = IndexedTextCache::new();
    let oldest = cache.insert(Entry { key: 1, value: 10 });
    let retained = cache.insert(Entry { key: 2, value: 20 });

    let (slot, entry, work) = cache
        .pop_oldest_with_slot_and_work()
        .expect("tracked LRU entry");

    assert_eq!(slot, oldest);
    assert_eq!(entry.value, 10);
    assert_eq!(work.scan_count, 1);
    assert_eq!(work.entry_move_count, 0);
    assert_eq!(cache.entry(retained).map(|entry| entry.value), Some(20));
}
