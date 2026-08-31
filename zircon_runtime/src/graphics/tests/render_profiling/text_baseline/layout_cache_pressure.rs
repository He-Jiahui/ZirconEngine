use std::{collections::BTreeSet, path::Path};

use crate::core::diagnostics::profiling::{
    PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE, ProfileCaptureConfig, export_report, reset_capture,
    start_capture, stop_capture, test_capture_lock,
};
use crate::text::cache::{DEFAULT_SHAPED_RUN_CACHE_CAPACITY, DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY};
use crate::ui::text::{UiTextLayoutRequest, UiTextMeasureCache};
use zircon_runtime_interface::{
    ProfileSnapshot,
    ui::{layout::UiFrame, surface::UiResolvedStyle},
};

use super::support::managed_output_root;
use super::{
    LABEL_COUNTS, MAX_SAMPLES, MEASURED_FRAMES, REPETITIONS, WARMUP_FRAMES, assert_profile_file,
    static_label_text_identity,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutCacheFrameSample {
    capacity: usize,
    entries: usize,
    hits: u64,
    misses: u64,
    lookup_candidates: u64,
    eviction_scans: u64,
    entry_moves: u64,
    evictions: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShapedRunCacheFrameSample {
    capacity: usize,
    entries: usize,
    hits: u64,
    misses: u64,
    lookup_candidates: u64,
    owned_key_allocation_bytes: u64,
    eviction_scans: u64,
    entry_moves: u64,
    inserts: u64,
    evictions: u64,
}

#[test]
fn layout_cache_pressure_profile_contract_matches_plan() {
    assert_eq!(LABEL_COUNTS, [1, 100, 1_000, 10_000]);
    assert_eq!(WARMUP_FRAMES, 60);
    assert_eq!(MEASURED_FRAMES, 300);
    assert_eq!(REPETITIONS, 3);
    assert_eq!(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY, 2_048);
    assert_eq!(DEFAULT_SHAPED_RUN_CACHE_CAPACITY, 1_024);
    assert!(MEASURED_FRAMES * 16 <= MAX_SAMPLES);
    assert_eq!(
        capture_config(10_000, 3, Path::new(r"E:\managed-text-profile")).session_id,
        "runtime-text-layout-cache-pressure-10000-r3"
    );
}

#[test]
#[ignore = "managed Windows text layout-cache profiling baseline"]
fn runtime_text_layout_cache_capacity_profile_baseline_exports_complete_matrix() {
    let _guard = test_capture_lock();
    let output_root = managed_output_root();

    for label_count in LABEL_COUNTS {
        let texts = (0..label_count)
            .map(|index| {
                let identity = static_label_text_identity(label_count, index);
                format!("L{identity:04}")
            })
            .collect::<Vec<_>>();
        let frames = (0..label_count)
            .map(|index| UiFrame::new(index as f32, 0.0, 64.0, 20.0))
            .collect::<Vec<_>>();
        let style = UiResolvedStyle::default();
        let mut cache = UiTextMeasureCache::default();

        for repetition in 1..=REPETITIONS {
            let (mut warm_layout, mut warm_shape) =
                resolve_layout_cache_frame(&mut cache, &texts, &frames, &style);
            for _ in 1..WARMUP_FRAMES {
                (warm_layout, warm_shape) =
                    resolve_layout_cache_frame(&mut cache, &texts, &frames, &style);
            }
            assert_layout_cache_capacity_contract(label_count, warm_layout);
            assert_shaped_run_cache_capacity_contract(label_count, warm_shape);

            start_capture(capture_config(label_count, repetition, &output_root));
            for _ in 0..MEASURED_FRAMES {
                crate::profile_frame!("runtime", "runtime_text.layout_cache_pressure");
                let (layout, shaped_runs) =
                    resolve_layout_cache_frame(&mut cache, &texts, &frames, &style);
                assert_layout_cache_capacity_contract(label_count, layout);
                assert_shaped_run_cache_capacity_contract(label_count, shaped_runs);
            }
            stop_capture();

            let report = export_report().expect("export text layout-cache pressure baseline");
            reset_capture();
            assert_complete_capture(label_count, &report.snapshot);
            for expected_file in [
                PROFILE_TIMELINE_NATIVE_FILE,
                PROFILE_TIMELINE_PERFETTO_FILE,
                PROFILE_HOTSPOTS_FILE,
                PROFILE_SUMMARY_FILE,
            ] {
                assert_profile_file(&report.files, expected_file);
            }
            assert!(Path::new(&report.export_dir).starts_with(&output_root));
        }
    }
}

fn resolve_layout_cache_frame(
    cache: &mut UiTextMeasureCache,
    texts: &[String],
    frames: &[UiFrame],
    style: &UiResolvedStyle,
) -> (LayoutCacheFrameSample, ShapedRunCacheFrameSample) {
    cache.begin_frame();
    {
        crate::profile_scope!(
            "runtime",
            "ui_text.layout_cache_pressure",
            "resolve_layouts"
        );
        for (text, frame) in texts.iter().zip(frames.iter().copied()) {
            let request = UiTextLayoutRequest::new(text, style, frame, None);
            let _ = cache.resolve_or_shape(&request);
        }
    }
    let report = cache.frame_layout_report();
    let shaped_runs = cache.frame_shaped_run_report();
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.hits",
        report.hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.misses",
        report.miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.entries",
        report.entry_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.lookup_candidates",
        report.lookup_candidate_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.eviction_scans",
        report.eviction_scan_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.entry_moves",
        report.entry_move_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.evictions",
        report.evicted_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_hits",
        shaped_runs.hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_entries",
        shaped_runs.entry_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_misses",
        shaped_runs.miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_lookup_candidates",
        shaped_runs.lookup_candidate_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_owned_key_allocation_bytes",
        shaped_runs.owned_key_allocation_bytes
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_eviction_scans",
        shaped_runs.eviction_scan_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_entry_moves",
        shaped_runs.entry_move_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_inserts",
        shaped_runs.insert_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_cache_pressure.shape_cache_evictions",
        shaped_runs.evicted_count
    );
    cache.finish_frame();
    (
        LayoutCacheFrameSample {
            capacity: report.capacity,
            entries: report.entry_count,
            hits: report.hit_count,
            misses: report.miss_count,
            lookup_candidates: report.lookup_candidate_count,
            eviction_scans: report.eviction_scan_count,
            entry_moves: report.entry_move_count,
            evictions: report.evicted_count,
        },
        ShapedRunCacheFrameSample {
            capacity: shaped_runs.capacity,
            entries: shaped_runs.entry_count,
            hits: shaped_runs.hit_count,
            misses: shaped_runs.miss_count,
            lookup_candidates: shaped_runs.lookup_candidate_count,
            owned_key_allocation_bytes: shaped_runs.owned_key_allocation_bytes,
            eviction_scans: shaped_runs.eviction_scan_count,
            entry_moves: shaped_runs.entry_move_count,
            inserts: shaped_runs.insert_count,
            evictions: shaped_runs.evicted_count,
        },
    )
}

fn assert_layout_cache_capacity_contract(label_count: usize, sample: LayoutCacheFrameSample) {
    assert_eq!(sample.capacity, DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY);
    if label_count <= sample.capacity {
        assert_eq!(sample.entries, label_count);
        assert_eq!(sample.hits, label_count as u64);
        assert_eq!(sample.misses, 0);
        assert_eq!(sample.lookup_candidates, label_count as u64);
        assert_eq!(sample.eviction_scans, 0);
        assert_eq!(sample.entry_moves, 0);
        assert_eq!(sample.evictions, 0);
    } else {
        assert_eq!(sample.entries, sample.capacity);
        assert_eq!(sample.hits, 0);
        assert_eq!(sample.misses, label_count as u64);
        assert_eq!(sample.lookup_candidates, 0);
        assert_eq!(sample.eviction_scans, label_count as u64);
        assert_eq!(sample.entry_moves, 0);
        assert_eq!(sample.evictions, label_count as u64);
    }
}

fn assert_shaped_run_cache_capacity_contract(
    label_count: usize,
    sample: ShapedRunCacheFrameSample,
) {
    let expected_entries = label_count.min(super::LARGE_LABEL_STABLE_TEXT_COUNT) + 1;
    assert_eq!(sample.capacity, DEFAULT_SHAPED_RUN_CACHE_CAPACITY);
    assert_eq!(sample.entries, expected_entries);
    assert!(expected_entries <= sample.capacity);
    if label_count <= DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY {
        // The persistent layout cache returns completed geometry before the layout engine needs a
        // shaped run. A zero shape-cache delta distinguishes this retained path from 10k capacity
        // pressure instead of inventing work below the layout-cache cap.
        assert_eq!(sample.hits, 0);
        assert_eq!(sample.lookup_candidates, 0);
    } else {
        assert!(
            sample.hits >= label_count as u64,
            "capacity pressure must reuse the warm shaped-run entries while layout keys churn"
        );
        assert!(
            sample.lookup_candidates >= label_count as u64,
            "every warm shaped-run cache hit must expose its bucket candidate work"
        );
    }
    assert_eq!(sample.misses, 0);
    assert_eq!(sample.owned_key_allocation_bytes, 0);
    assert_eq!(sample.eviction_scans, 0);
    assert_eq!(sample.entry_moves, 0);
    assert_eq!(sample.inserts, 0);
    assert_eq!(sample.evictions, 0);
}

fn capture_config(
    label_count: usize,
    repetition: usize,
    output_root: &Path,
) -> ProfileCaptureConfig {
    ProfileCaptureConfig {
        session_id: format!("runtime-text-layout-cache-pressure-{label_count}-r{repetition}"),
        output_root: output_root.to_string_lossy().into_owned(),
        max_frames: MEASURED_FRAMES,
        max_spans: MAX_SAMPLES,
        max_counters: MAX_SAMPLES,
        include_perfetto: true,
        ..ProfileCaptureConfig::default()
    }
}

fn assert_complete_capture(label_count: usize, snapshot: &ProfileSnapshot) {
    assert_eq!(snapshot.frames.len(), MEASURED_FRAMES);
    let spans = snapshot
        .spans
        .iter()
        .filter(|span| {
            span.category == "ui_text.layout_cache_pressure" && span.name == "resolve_layouts"
        })
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), MEASURED_FRAMES);
    assert_eq!(
        spans
            .iter()
            .filter_map(|span| span.frame_index)
            .collect::<BTreeSet<_>>()
            .len(),
        MEASURED_FRAMES
    );

    let expected_hits = if label_count <= DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY {
        label_count as f64
    } else {
        0.0
    };
    let expected_misses = if label_count <= DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY {
        0.0
    } else {
        label_count as f64
    };
    let expected_entries = label_count.min(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY) as f64;
    let expected_lookup_candidates = if label_count <= DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY {
        label_count as f64
    } else {
        0.0
    };
    let expected_evictions = if label_count <= DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY {
        0.0
    } else {
        label_count as f64
    };
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.hits",
        expected_hits,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.misses",
        expected_misses,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.entries",
        expected_entries,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.lookup_candidates",
        expected_lookup_candidates,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.eviction_scans",
        expected_evictions,
    );
    assert_counter_equals(snapshot, "ui_text.layout_cache_pressure.entry_moves", 0.0);
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.evictions",
        expected_evictions,
    );

    let expected_shape_entries = label_count
        .min(super::LARGE_LABEL_STABLE_TEXT_COUNT)
        .saturating_add(1) as f64;
    let shape_cache_requires_layout_reuse = label_count > DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY;
    if shape_cache_requires_layout_reuse {
        assert_counter_at_least(
            snapshot,
            "ui_text.layout_cache_pressure.shape_cache_hits",
            label_count as f64,
        );
    } else {
        assert_counter_equals(
            snapshot,
            "ui_text.layout_cache_pressure.shape_cache_hits",
            0.0,
        );
    }
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_entries",
        expected_shape_entries,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_misses",
        0.0,
    );
    if shape_cache_requires_layout_reuse {
        assert_counter_at_least(
            snapshot,
            "ui_text.layout_cache_pressure.shape_cache_lookup_candidates",
            label_count as f64,
        );
    } else {
        assert_counter_equals(
            snapshot,
            "ui_text.layout_cache_pressure.shape_cache_lookup_candidates",
            0.0,
        );
    }
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_owned_key_allocation_bytes",
        0.0,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_eviction_scans",
        0.0,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_entry_moves",
        0.0,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_inserts",
        0.0,
    );
    assert_counter_equals(
        snapshot,
        "ui_text.layout_cache_pressure.shape_cache_evictions",
        0.0,
    );
}

fn assert_counter_equals(snapshot: &ProfileSnapshot, name: &str, expected: f64) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_eq!(samples.len(), MEASURED_FRAMES);
    assert_eq!(
        samples
            .iter()
            .filter_map(|counter| counter.frame_index)
            .collect::<BTreeSet<_>>()
            .len(),
        MEASURED_FRAMES
    );
    assert!(
        samples.iter().all(|counter| counter.value == expected),
        "layout-cache pressure baseline requires `{name}` to equal {expected}"
    );
}

fn assert_counter_at_least(snapshot: &ProfileSnapshot, name: &str, minimum: f64) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_eq!(samples.len(), MEASURED_FRAMES);
    assert_eq!(
        samples
            .iter()
            .filter_map(|counter| counter.frame_index)
            .collect::<BTreeSet<_>>()
            .len(),
        MEASURED_FRAMES
    );
    assert!(
        samples.iter().all(|counter| counter.value >= minimum),
        "layout-cache pressure baseline requires `{name}` to remain at least {minimum}"
    );
}
