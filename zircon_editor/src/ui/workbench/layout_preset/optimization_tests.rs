use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{
    LayoutPreset, LayoutPresetName, LayoutPresetPersistenceEntry, LayoutPresetPersistenceStore,
    LayoutPresetScope,
};
use crate::ui::workbench::layout::MainPageId;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 512;
const ENTRY_COUNT: usize = 1_024;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_entries() -> Vec<LayoutPresetPersistenceEntry> {
    (0..ENTRY_COUNT)
        .map(|index| LayoutPresetPersistenceEntry {
            scope: LayoutPresetScope::new(
                format!("user-{index:04}"),
                MainPageId::new(format!("page-{index:04}")),
            ),
            preset: LayoutPreset::authoring(),
        })
        .collect()
}

fn measure_samples(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed()
        })
        .collect()
}

fn legacy_insert(
    entries: &mut Vec<LayoutPresetPersistenceEntry>,
    entry: LayoutPresetPersistenceEntry,
) {
    if let Some(existing) = entries
        .iter_mut()
        .find(|existing| existing.scope == entry.scope)
    {
        existing.preset = entry.preset;
    } else {
        entries.push(entry);
        entries.sort_by(|left, right| left.scope.cmp(&right.scope));
    }
}

fn indexed_insert(
    entries: &mut Vec<LayoutPresetPersistenceEntry>,
    entry: LayoutPresetPersistenceEntry,
) {
    match entries.binary_search_by(|existing| existing.scope.cmp(&entry.scope)) {
        Ok(index) => entries[index].preset = entry.preset,
        Err(index) => entries.insert(index, entry),
    }
}

fn legacy_lookup(
    entries: &[LayoutPresetPersistenceEntry],
    scope: &LayoutPresetScope,
) -> Option<usize> {
    entries.iter().position(|entry| &entry.scope == scope)
}

fn indexed_lookup(
    entries: &[LayoutPresetPersistenceEntry],
    scope: &LayoutPresetScope,
) -> Option<usize> {
    entries
        .binary_search_by(|entry| entry.scope.cmp(scope))
        .ok()
}

#[test]
fn editor13_preset_index_preserves_sorted_upsert_and_restore_behavior() {
    let mut store = LayoutPresetPersistenceStore::default();
    let page_id = MainPageId::new("scene:main");
    let later = LayoutPresetScope::new("zeta", page_id.clone());
    let earlier = LayoutPresetScope::new("alpha", page_id.clone());
    store.persist_layout(later.clone(), LayoutPreset::focus());
    store.persist_layout(earlier.clone(), LayoutPreset::authoring());
    store.persist_layout(later.clone(), LayoutPreset::debug());

    assert!(store
        .entries()
        .windows(2)
        .all(|entries| entries[0].scope < entries[1].scope));
    assert_eq!(store.entries().len(), 2);
    assert_eq!(
        store.restore_layout(&later).preset().name,
        LayoutPresetName::Debug
    );
    assert_eq!(
        store.restore_layout(&earlier).preset().name,
        LayoutPresetName::Authoring
    );
}

#[test]
fn editor13_preset_source_uses_sorted_binary_search_boundaries() {
    let source = include_str!("../layout_preset.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .expect("tests follow production")
        .0;
    assert_eq!(production.matches("binary_search_by").count(), 2);
    assert!(!production.contains("entries.iter_mut().find"));
    assert!(!production.contains("entries\n                .sort_by"));
    assert!(!production.contains("entries\n            .iter()\n            .find"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor13_binary_search_preset_insert_bench() {
    let fixture = fixture_entries();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            let mut entries = fixture.clone();
            let entry = entries[ENTRY_COUNT / 2].clone();
            black_box(legacy_insert(&mut entries, entry));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            let mut entries = fixture.clone();
            let entry = entries[ENTRY_COUNT / 2].clone();
            black_box(indexed_insert(&mut entries, entry));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);

    println!(
        "EDITOR13_BINARY_SEARCH_PRESET_INSERT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} lookup_candidate_checks={}->{} full_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        ENTRY_COUNT,
        (ENTRY_COUNT as f64).log2().ceil() as usize,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 85,
        "optimized p95 should be at most 85% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor13_binary_search_preset_restore_bench() {
    let entries = fixture_entries();
    let target = &entries[ENTRY_COUNT - 1].scope;
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_lookup(&entries, target));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(indexed_lookup(&entries, target));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);

    println!(
        "EDITOR13_BINARY_SEARCH_PRESET_RESTORE_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} entries={} lookup_candidate_checks={}->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        ENTRY_COUNT,
        ENTRY_COUNT,
        (ENTRY_COUNT as f64).log2().ceil() as usize,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 50,
        "optimized p95 should be at most 50% of legacy p95"
    );
}
