use std::ffi::c_void;
use std::hint::black_box;
use std::mem::{MaybeUninit, size_of};
use std::time::Instant;

use super::*;

const WARM_UP_COUNT: usize = 3;
const SAMPLE_COUNT: usize = 31;
const DENSE_OBJECT_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];
const WRAPPED_LINE_COUNTS: [usize; 3] = [1, 100, 1_000];
const INLINE_IMAGE: &str = "<img src=\"res://icons/profile.png\" width=\"12\" height=\"14\">";

#[derive(Clone, Copy, Debug)]
enum ProfileLane {
    DenseLtr,
    DenseRtl,
    DenseVerticalRl,
    WrappedLines,
}

impl ProfileLane {
    const fn name(self) -> &'static str {
        match self {
            Self::DenseLtr => "dense_ltr",
            Self::DenseRtl => "dense_rtl",
            Self::DenseVerticalRl => "dense_vertical_rl",
            Self::WrappedLines => "wrapped_lines",
        }
    }

    const fn counts(self) -> &'static [usize] {
        match self {
            Self::WrappedLines => &WRAPPED_LINE_COUNTS,
            _ => &DENSE_OBJECT_COUNTS,
        }
    }
}

struct ProfileFixture {
    extract: UiRenderExtract,
    viewport: UVec2,
    inline_count: usize,
    line_count: usize,
}

#[derive(Clone, Copy)]
struct InlineWork {
    inline_run_count: usize,
    line_probe_count: usize,
    line_run_probe_count: usize,
    prefix_grapheme_count: usize,
    prefix_advance_count: usize,
    paint_frame_match_count: usize,
    paint_frame_mismatch_count: usize,
}

#[test]
#[ignore = "release-only 31-sample rich inline geometry baseline"]
fn rich_inline_geometry_release_baseline() {
    assert!(
        !cfg!(debug_assertions),
        "run the rich inline geometry baseline with a release test profile"
    );
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();

    for lane in [
        ProfileLane::DenseLtr,
        ProfileLane::DenseRtl,
        ProfileLane::DenseVerticalRl,
        ProfileLane::WrappedLines,
    ] {
        for &scale in lane.counts() {
            run_lane(lane, scale);
        }
    }
}

fn run_lane(lane: ProfileLane, scale: usize) {
    let fixture = build_fixture(lane, scale);
    let work = capture_inline_work(&fixture);
    assert_eq!(work.inline_run_count, fixture.inline_count);
    assert_eq!(work.paint_frame_match_count, fixture.inline_count);
    assert_eq!(work.paint_frame_mismatch_count, 0);

    for _ in 0..WARM_UP_COUNT {
        assert_eq!(plan_fixture(&fixture), fixture.inline_count);
    }

    let mut samples_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut rss_delta_bytes = Vec::with_capacity(SAMPLE_COUNT);
    let mut checksum = 0_usize;
    for _ in 0..SAMPLE_COUNT {
        let rss_before = current_rss_bytes();
        let started = Instant::now();
        checksum = checksum.saturating_add(plan_fixture(black_box(&fixture)));
        samples_ns.push(started.elapsed().as_nanos());
        let rss_after = current_rss_bytes();
        rss_delta_bytes.push(rss_after as i128 - rss_before as i128);
    }

    assert_eq!(checksum, fixture.inline_count.saturating_mul(SAMPLE_COUNT));
    let p50_ns = nearest_rank(&samples_ns, 50);
    let p95_ns = nearest_rank(&samples_ns, 95);
    let p99_ns = nearest_rank(&samples_ns, 99);
    let lane_name = lane.name();
    let inline_count = fixture.inline_count;
    let line_count = fixture.line_count;
    let InlineWork {
        inline_run_count,
        line_probe_count,
        line_run_probe_count,
        prefix_grapheme_count,
        prefix_advance_count,
        paint_frame_match_count,
        paint_frame_mismatch_count,
    } = work;
    eprintln!(
        "RUNTIME_TEXT_RICH_INLINE_GEOMETRY_BASELINE_V1 build=release lane={lane_name} scale={scale} lines={line_count} inline_objects={inline_count} warm_up_count={WARM_UP_COUNT} sample_count={SAMPLE_COUNT} inline_run_count={inline_run_count} line_probe_count={line_probe_count} line_run_probe_count={line_run_probe_count} prefix_grapheme_count={prefix_grapheme_count} prefix_advance_count={prefix_advance_count} paint_frame_match_count={paint_frame_match_count} paint_frame_mismatch_count={paint_frame_mismatch_count} p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} samples_ns={samples_ns:?} rss_delta_bytes={rss_delta_bytes:?}"
    );
}

fn build_fixture(lane: ProfileLane, scale: usize) -> ProfileFixture {
    let (markup, mut style, frame, viewport, expected_lines) = match lane {
        ProfileLane::DenseLtr => {
            let extent = scale as f32 * 24.0 + 32.0;
            (
                dense_markup("a", scale),
                rich_profile_style(),
                UiFrame::new(8.0, 8.0, extent, 24.0),
                UVec2::new(extent.ceil() as u32 + 16, 48),
                1,
            )
        }
        ProfileLane::DenseRtl => {
            let extent = scale as f32 * 24.0 + 32.0;
            let mut style = rich_profile_style();
            style.text_direction = UiTextDirection::RightToLeft;
            (
                dense_markup("א", scale),
                style,
                UiFrame::new(8.0, 8.0, extent, 24.0),
                UVec2::new(extent.ceil() as u32 + 16, 48),
                1,
            )
        }
        ProfileLane::DenseVerticalRl => {
            let extent = scale as f32 * 24.0 + 32.0;
            let mut style = rich_profile_style();
            style.text_writing_mode = UiTextWritingMode::VerticalRl;
            (
                dense_markup("甲", scale),
                style,
                UiFrame::new(8.0, 8.0, 32.0, extent),
                UVec2::new(64, extent.ceil() as u32 + 16),
                1,
            )
        }
        ProfileLane::WrappedLines => {
            let height = scale as f32 * 24.0 + 16.0;
            (
                wrapped_markup(scale),
                rich_profile_style(),
                UiFrame::new(8.0, 8.0, 64.0, height),
                UVec2::new(96, height.ceil() as u32 + 16),
                scale,
            )
        }
    };
    style.wrap = UiTextWrap::None;
    let layout = layout_text(&markup, &style, frame, None);
    let inline_count = layout
        .lines
        .iter()
        .flat_map(|line| &line.runs)
        .filter(|run| run.text == "\u{fffc}")
        .count();
    assert_eq!(inline_count, scale);
    assert_eq!(layout.lines.len(), expected_lines);

    ProfileFixture {
        extract: UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-release-profile"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(614),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        viewport,
        inline_count,
        line_count: expected_lines,
    }
}

fn rich_profile_style() -> UiResolvedStyle {
    UiResolvedStyle {
        foreground_color: Some("#ffffff".to_owned()),
        font_size: 12.0,
        line_height: 16.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::HtmlSubsetV1,
        ..UiResolvedStyle::default()
    }
}

fn dense_markup(prefix: &str, count: usize) -> String {
    let mut markup = String::with_capacity((prefix.len() + INLINE_IMAGE.len()) * count);
    for _ in 0..count {
        markup.push_str(prefix);
        markup.push_str(INLINE_IMAGE);
    }
    markup
}

fn wrapped_markup(line_count: usize) -> String {
    let mut markup = String::with_capacity((1 + INLINE_IMAGE.len() + 1) * line_count);
    for line_index in 0..line_count {
        if line_index > 0 {
            markup.push('\n');
        }
        markup.push('a');
        markup.push_str(INLINE_IMAGE);
    }
    markup
}

fn capture_inline_work(fixture: &ProfileFixture) -> InlineWork {
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = format!("rich-inline-geometry-{}", fixture.inline_count);
    config.max_spans = 32;
    config.max_counters = 128;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);
    assert_eq!(plan_fixture(fixture), fixture.inline_count);
    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(!crate::core::diagnostics::profiling::reset_capture().active);

    InlineWork {
        inline_run_count: counter(&snapshot, "rich_inline_run_count"),
        line_probe_count: counter(&snapshot, "rich_inline_line_probe_count"),
        line_run_probe_count: counter(&snapshot, "rich_inline_line_run_probe_count"),
        prefix_grapheme_count: counter(&snapshot, "rich_inline_prefix_grapheme_count"),
        prefix_advance_count: counter(&snapshot, "rich_inline_prefix_advance_count"),
        paint_frame_match_count: counter(&snapshot, "rich_inline_paint_frame_match_count"),
        paint_frame_mismatch_count: counter(&snapshot, "rich_inline_paint_frame_mismatch_count"),
    }
}

fn counter(snapshot: &crate::core::diagnostics::profiling::ProfileSnapshot, name: &str) -> usize {
    snapshot
        .counters
        .iter()
        .find(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value as usize)
        .unwrap_or_else(|| panic!("missing rich inline profile counter {name}"))
}

fn plan_fixture(fixture: &ProfileFixture) -> usize {
    let plan = plan_screen_space_ui_batches(&fixture.extract, fixture.viewport);
    black_box(plan).images.len()
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
