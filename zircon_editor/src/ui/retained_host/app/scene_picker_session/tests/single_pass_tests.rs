use std::hint::black_box;
use std::time::Instant;

use super::{normalized_window_offset, scene_open_query_window, ScenePickerEntry};

const WINDOW: usize = 12;
const SAMPLE_PAIRS: usize = 17;
const ENTRY_COUNT: usize = 65_536;

fn entries(count: usize) -> Vec<ScenePickerEntry> {
    (0..count)
        .map(|index| ScenePickerEntry {
            command_id: format!("scene-{index}"),
            scene_uri: format!("res://levels/scene-{index:05}.scene.toml"),
        })
        .collect()
}

fn legacy_scene_open_query_window<'a>(
    entries: &'a [ScenePickerEntry],
    query: &str,
    requested_offset: usize,
) -> (usize, usize, Vec<&'a ScenePickerEntry>) {
    let normalized_query = query.trim().to_ascii_lowercase();
    let total_match_count = entries
        .iter()
        .filter(|entry| entry.matches_query(&normalized_query))
        .count();
    let window_offset = normalized_window_offset(requested_offset, total_match_count);
    let visible = entries
        .iter()
        .filter(|entry| entry.matches_query(&normalized_query))
        .skip(window_offset)
        .take(WINDOW)
        .collect();
    (total_match_count, window_offset, visible)
}

#[test]
fn optimization_batch_20260915_editor787_scene_picker_single_pass_preserves_windows() {
    let source_entries = entries(37);
    for (query, requested_offset) in [("", 0), ("scene-", 12), ("scene-3", 0), ("missing", 0)] {
        let expected = legacy_scene_open_query_window(&source_entries, query, requested_offset);
        let actual = scene_open_query_window(&source_entries, query, requested_offset);
        assert_eq!(actual.total_match_count, expected.0, "query={query}");
        assert_eq!(actual.window_offset, expected.1, "query={query}");
        assert_eq!(
            actual
                .entries
                .iter()
                .map(|entry| entry.command_id.as_str())
                .collect::<Vec<_>>(),
            expected
                .2
                .iter()
                .map(|entry| entry.command_id.as_str())
                .collect::<Vec<_>>(),
            "query={query} offset={requested_offset}"
        );
    }

    let fallback = scene_open_query_window(&source_entries, "scene-", 10_000);
    assert_eq!(fallback.total_match_count, 37);
    assert_eq!(fallback.window_offset, 36);
    assert_eq!(fallback.entries.len(), 1);
    assert_eq!(fallback.entries[0].command_id, "scene-36");
}

#[test]
fn optimization_batch_20260915_editor787_scene_picker_single_pass_keeps_bounded_storage() {
    let source = include_str!("../../scene_picker_session.rs");
    let production = source.split("#[cfg(test)]").next().expect("production");
    assert_eq!(production.matches("for entry in entries {").count(), 1);
    assert!(production.contains("Vec::with_capacity(SCENE_PICKER_WINDOW_ENTRIES)"));
    assert_eq!(
        production
            .matches("Vec::with_capacity(SCENE_PICKER_WINDOW_ENTRIES)")
            .count(),
        2
    );
    assert!(!production.contains(".count()"));
}

fn measure_nanos(run: impl FnOnce() -> u64) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos().max(1)
}

fn legacy_scan(entries: &[ScenePickerEntry]) -> u64 {
    let (_, offset, visible) = legacy_scene_open_query_window(entries, "scene-", 1_024);
    (visible.len() as u64).wrapping_add(offset as u64)
}

fn single_pass(entries: &[ScenePickerEntry]) -> u64 {
    let result = scene_open_query_window(entries, "scene-", 1_024);
    (result.entries.len() as u64).wrapping_add(result.window_offset as u64)
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260915_editor787_scene_picker_single_pass_p95() {
    let source_entries = entries(ENTRY_COUNT);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut single_pass_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_nanos(|| legacy_scan(black_box(&source_entries))));
            single_pass_samples.push(measure_nanos(|| single_pass(black_box(&source_entries))));
        } else {
            single_pass_samples.push(measure_nanos(|| single_pass(black_box(&source_entries))));
            legacy_samples.push(measure_nanos(|| legacy_scan(black_box(&source_entries))));
        }
    }
    let legacy_p95 = percentile_95(&mut legacy_samples);
    let single_pass_p95 = percentile_95(&mut single_pass_samples);
    println!(
        "EDITOR787_SCENE_PICKER_SINGLE_PASS_WINDOW_BENCH_V1 entries={ENTRY_COUNT} window={WINDOW} \
         legacy_scans=2 single_pass_scans=1 legacy_p95_ns={legacy_p95} \
         single_pass_p95_ns={single_pass_p95}"
    );
    assert!(legacy_p95 > 0);
    assert!(single_pass_p95 > 0);
    assert_eq!(WINDOW, 12);
}
