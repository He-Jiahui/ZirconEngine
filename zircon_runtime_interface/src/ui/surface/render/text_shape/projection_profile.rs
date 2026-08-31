use std::ffi::c_void;
use std::hint::black_box;
use std::mem::{MaybeUninit, size_of};
use std::time::Instant;

use crate::ui::layout::UiFrame;

use super::super::{UiResolvedTextLine, UiResolvedTextRun};
use super::{
    UiResolvedTextLayout, UiTextDirection, UiTextRange, UiTextRunKind, UiTextWritingMode,
    text_paint_runs_from_resolved_layout,
};

const WARM_UP_COUNT: usize = 3;
const SAMPLE_COUNT: usize = 31;
const DENSE_RUN_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];

#[test]
#[ignore = "release-only 31-sample paint projection baseline"]
fn resolved_text_paint_run_projection_dense_run_baseline() {
    assert!(
        !cfg!(debug_assertions),
        "run the paint projection baseline with a release test profile"
    );

    for run_count in DENSE_RUN_COUNTS {
        let layout = dense_styled_layout(run_count);
        for _ in 0..WARM_UP_COUNT {
            assert_eq!(project_dense_layout(&layout), run_count);
        }

        let mut samples_ns = Vec::with_capacity(SAMPLE_COUNT);
        let mut rss_delta_bytes = Vec::with_capacity(SAMPLE_COUNT);
        let mut checksum = 0_usize;
        for _ in 0..SAMPLE_COUNT {
            let rss_before = current_rss_bytes();
            let started = Instant::now();
            checksum = checksum.saturating_add(project_dense_layout(black_box(&layout)));
            samples_ns.push(started.elapsed().as_nanos());
            let rss_after = current_rss_bytes();
            rss_delta_bytes.push(rss_after as i128 - rss_before as i128);
        }

        assert_eq!(checksum, run_count.saturating_mul(SAMPLE_COUNT));
        let p50_ns = nearest_rank(&samples_ns, 50);
        let p95_ns = nearest_rank(&samples_ns, 95);
        let p99_ns = nearest_rank(&samples_ns, 99);
        let implied_full_line_grapheme_visits = run_count.saturating_mul(run_count);
        eprintln!(
            "RUNTIME_TEXT_PAINT_RUN_PROJECTION_BASELINE_V1 build=release writing=horizontal_ltr lines=1 runs={run_count} graphemes={run_count} warm_up_count={WARM_UP_COUNT} sample_count={SAMPLE_COUNT} implied_full_line_grapheme_visits={implied_full_line_grapheme_visits} p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} samples_ns={samples_ns:?} rss_delta_bytes={rss_delta_bytes:?}"
        );
    }
}

fn dense_styled_layout(run_count: usize) -> UiResolvedTextLayout {
    let text = "a".repeat(run_count);
    let runs = (0..run_count)
        .map(|index| UiResolvedTextRun {
            kind: match index % 4 {
                0 => UiTextRunKind::Plain,
                1 => UiTextRunKind::Strong,
                2 => UiTextRunKind::Emphasis,
                _ => UiTextRunKind::Code,
            },
            text: "a".to_owned(),
            source_range: UiTextRange {
                start: index,
                end: index + 1,
            },
            visual_range: UiTextRange {
                start: index,
                end: index + 1,
            },
            direction: UiTextDirection::LeftToRight,
        })
        .collect();
    let extent = run_count as f32;

    UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: 16.0,
        line_height: 20.0,
        measured_width: extent,
        measured_height: 20.0,
        source_range: UiTextRange {
            start: 0,
            end: run_count,
        },
        lines: vec![UiResolvedTextLine {
            text,
            frame: UiFrame::new(0.0, 0.0, extent, 20.0),
            placement_frame: UiFrame::new(0.0, 0.0, extent, 20.0),
            source_range: UiTextRange {
                start: 0,
                end: run_count,
            },
            visual_range: UiTextRange {
                start: 0,
                end: run_count,
            },
            measured_width: extent,
            glyph_advances: vec![1.0; run_count],
            baseline: 16.0,
            direction: UiTextDirection::LeftToRight,
            runs,
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    }
}

fn project_dense_layout(layout: &UiResolvedTextLayout) -> usize {
    let no_string = None;
    let runs = text_paint_runs_from_resolved_layout(
        layout, &no_string, &no_string, &no_string, 400, 16.0, 20.0,
    );
    black_box(runs).len()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = percentile.saturating_mul(sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

#[repr(C)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
}

#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(
        process: *mut c_void,
        counters: *mut ProcessMemoryCounters,
        size: u32,
    ) -> i32;
}

fn current_rss_bytes() -> usize {
    let mut counters = MaybeUninit::<ProcessMemoryCounters>::zeroed();
    let counters_ptr = counters.as_mut_ptr();
    // SAFETY: PROCESS_MEMORY_COUNTERS is initialized with its ABI size and both pointers remain
    // valid for the duration of the OS call.
    unsafe {
        (*counters_ptr).cb = size_of::<ProcessMemoryCounters>() as u32;
        assert_ne!(
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                counters_ptr,
                size_of::<ProcessMemoryCounters>() as u32,
            ),
            0,
            "GetProcessMemoryInfo failed"
        );
        counters.assume_init().working_set_size
    }
}
