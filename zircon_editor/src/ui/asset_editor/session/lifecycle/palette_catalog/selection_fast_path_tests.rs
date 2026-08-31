use std::hint::black_box;
use std::time::Instant;

use super::selected_palette_entry_index;
use crate::ui::asset_editor::palette::{UiAssetPaletteEntry, UiAssetPaletteEntryKind};

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 512;

fn entry(index: usize, prefix: &str) -> UiAssetPaletteEntry {
    UiAssetPaletteEntry {
        label: format!("Component / {prefix}{index:04}"),
        kind: UiAssetPaletteEntryKind::Component {
            component: format!("{prefix}{index:04}"),
        },
    }
}

#[test]
fn optimization_batch_20260826bt_palette_selection_index_fast_path_preserves_stable_index() {
    let entries = (0..8)
        .map(|index| entry(index, "component-"))
        .collect::<Vec<_>>();
    let selected = entries[6].clone();

    assert_eq!(
        selected_palette_entry_index(&entries, &selected, Some(6)),
        Some(6)
    );
    assert_eq!(
        selected_palette_entry_index(&entries, &selected, Some(99)),
        Some(6)
    );
}

#[test]
fn optimization_batch_20260826bt_palette_selection_index_fast_path_preserves_reorder_fallback() {
    let mut entries = (0..8)
        .map(|index| entry(index, "component-"))
        .collect::<Vec<_>>();
    let selected = entries[6].clone();
    entries.swap(1, 6);

    assert_eq!(
        selected_palette_entry_index(&entries, &selected, Some(6)),
        Some(1)
    );
    let source = include_str!("../palette_catalog.rs");
    let helper = source
        .split_once("fn selected_palette_entry_index(")
        .unwrap()
        .1
        .split_once("pub(super) fn reconcile_palette_catalog_selection")
        .unwrap()
        .0;
    assert!(
        helper.find("entries.get(previous_index)").unwrap() < helper.find(".position(").unwrap()
    );
}

fn run_scan_workload(entries: &[UiAssetPaletteEntry], selected: &UiAssetPaletteEntry) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(entries.iter().position(|entry| entry == selected));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_fast_workload(
    entries: &[UiAssetPaletteEntry],
    selected: &UiAssetPaletteEntry,
    previous_index: usize,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(selected_palette_entry_index(
            entries,
            selected,
            Some(previous_index),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128], numerator: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * numerator).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[test]
#[ignore = "release performance gate; managed validation only"]
fn optimization_batch_20260826bt_palette_selection_index_fast_path_p95() {
    let prefix = "palette-selection-shared-prefix/".repeat(20);
    let entries = (0..ENTRY_COUNT)
        .map(|index| entry(index, &prefix))
        .collect::<Vec<_>>();
    let selected_index = ENTRY_COUNT - 1;
    let selected = entries[selected_index].clone();
    let mut scan_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut fast_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            scan_samples.push(run_scan_workload(&entries, &selected));
            fast_samples.push(run_fast_workload(&entries, &selected, selected_index));
        } else {
            fast_samples.push(run_fast_workload(&entries, &selected, selected_index));
            scan_samples.push(run_scan_workload(&entries, &selected));
        }
    }

    let scan_p50 = percentile(&mut scan_samples.clone(), 50);
    let scan_p95 = percentile(&mut scan_samples, 95);
    let fast_p50 = percentile(&mut fast_samples.clone(), 50);
    let fast_p95 = percentile(&mut fast_samples, 95);
    println!(
        "EDITOR01_PALETTE_SELECTION_INDEX_FAST_PATH_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} scan_p50_ns={scan_p50} scan_p95_ns={scan_p95} fast_p50_ns={fast_p50} fast_p95_ns={fast_p95} comparisons_before={} comparisons_after={HIT_COUNT}",
        ENTRY_COUNT * HIT_COUNT
    );
    assert!(
        fast_p95 * 100 <= scan_p95 * 10,
        "index fast-path P95 must be at least 90% below full scan: scan={scan_p95}ns fast={fast_p95}ns"
    );
}
