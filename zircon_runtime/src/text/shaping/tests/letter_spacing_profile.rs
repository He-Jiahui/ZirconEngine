use std::hint::black_box;
use std::mem::size_of;
use std::time::Instant;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::cache::{ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheReport};
use crate::text::{
    BackendShapeRequest, OpenTypeFeature, ShapedGlyph, ShapedGlyphRun, TextOrientation, TextRange,
    TextStyle, VerticalMode,
};

use super::super::shape_text;
use super::test_style;

const WARM_UP_COUNT: usize = 3;
const SAMPLE_COUNT: usize = 31;
const CLUSTER_COUNTS: [usize; 3] = [32, 256, 4_096];
const TRACKING_PX: f32 = 0.75;
const ALTERNATING_SPAN_CLUSTERS: usize = 16;
const GLYPH_WRAP_WIDTH: f32 = 320.0;

#[test]
#[ignore = "managed release-only 31-sample letter-spacing candidate baseline; no machine-time acceptance threshold"]
fn letter_spacing_candidate_release_baseline() {
    assert!(
        !cfg!(debug_assertions),
        "run the letter-spacing baseline with a release test profile"
    );
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();

    for workload in Workload::ALL {
        for cluster_count in CLUSTER_COUNTS {
            for span_mode in SpanMode::ALL {
                let fixture = ProfileFixture::new(workload, span_mode, cluster_count);
                let zero_cache = capture_current_cache_baseline(&fixture);
                run_lane(&fixture, TrackingMode::Zero, Some(zero_cache));
                run_lane(&fixture, TrackingMode::Candidate, None);
            }
        }
    }
}

fn run_lane(fixture: &ProfileFixture, tracking_mode: TrackingMode, cache_work: Option<CacheWork>) {
    for _ in 0..WARM_UP_COUNT {
        black_box(profile_fixture(fixture, tracking_mode));
    }

    let work = capture_backend_work(fixture, tracking_mode);
    let expected = profile_fixture(fixture, tracking_mode);
    let expected_signature = expected.signature();
    let mut samples_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut rss_delta_bytes = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let rss_before = current_rss_bytes();
        let started = Instant::now();
        let result = black_box(profile_fixture(black_box(fixture), tracking_mode));
        samples_ns.push(started.elapsed().as_nanos());
        let rss_after = current_rss_bytes();
        rss_delta_bytes.push(rss_after as i128 - rss_before as i128);
        assert_eq!(result.signature(), expected_signature);
    }

    let p50_ns = nearest_rank(&samples_ns, 50);
    let p95_ns = nearest_rank(&samples_ns, 95);
    let p99_ns = nearest_rank(&samples_ns, 99);
    let workload = fixture.workload.label();
    let span_mode = fixture.span_mode.label();
    let tracking = tracking_mode.label();
    let requested_clusters = fixture.requested_cluster_count;
    let requested_bytes = fixture.requested_bytes;
    let ProfileResult {
        glyph_count,
        cluster_count,
        adjusted_gap_count,
        bypassed_run_count,
        glyph_wrap_line_count,
        measured_width,
        estimated_output_bytes,
        routes,
    } = expected;
    let route = routes.label();
    let backend_shape_calls = work.backend_shape_calls;
    let shaping_request_count = work.shaping_request_count;
    let (cache_cold_hits, cache_cold_misses, cache_warm_hits, cache_warm_misses) = cache_work
        .map(|work| {
            (
                work.cold.hit_count,
                work.cold.miss_count,
                work.warm.hit_count,
                work.warm.miss_count,
            )
        })
        .unwrap_or_default();

    eprintln!(
        "RUNTIME_TEXT_LETTER_SPACING_CANDIDATE_BASELINE_V1 build=release workload={workload} span_mode={span_mode} tracking={tracking} tracking_px={TRACKING_PX} requested_clusters={requested_clusters} requested_bytes={requested_bytes} warm_up_count={WARM_UP_COUNT} sample_count={SAMPLE_COUNT} p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} samples_ns={samples_ns:?} rss_delta_bytes={rss_delta_bytes:?} backend_shape_calls={backend_shape_calls} shaping_request_count={shaping_request_count} glyph_count={glyph_count} cluster_count={cluster_count} adjusted_gap_count={adjusted_gap_count} bypassed_run_count={bypassed_run_count} glyph_wrap_line_count={glyph_wrap_line_count} measured_width={measured_width} estimated_output_bytes={estimated_output_bytes} allocation_count_available=false route={route} alternate_backend_calls_instrumented=false cache_cold_hits={cache_cold_hits} cache_cold_misses={cache_cold_misses} cache_warm_hits={cache_warm_hits} cache_warm_misses={cache_warm_misses} candidate_cache_identity_supported=false"
    );
}

#[derive(Clone, Copy, Debug)]
enum Workload {
    LatinLigature,
    Cjk,
    CombiningMark,
    EmojiZwj,
    ArabicRtl,
    MixedBidi,
    VerticalCjk,
}

impl Workload {
    const ALL: [Self; 7] = [
        Self::LatinLigature,
        Self::Cjk,
        Self::CombiningMark,
        Self::EmojiZwj,
        Self::ArabicRtl,
        Self::MixedBidi,
        Self::VerticalCjk,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::LatinLigature => "latin_ligature",
            Self::Cjk => "cjk",
            Self::CombiningMark => "combining_mark",
            Self::EmojiZwj => "emoji_zwj",
            Self::ArabicRtl => "arabic_rtl",
            Self::MixedBidi => "mixed_bidi",
            Self::VerticalCjk => "vertical_cjk",
        }
    }

    const fn seed(self) -> &'static str {
        match self {
            Self::LatinLigature => "office affinity ",
            Self::Cjk | Self::VerticalCjk => "漢字仮名交じり文",
            Self::CombiningMark => "a\u{0301}e\u{0308}",
            Self::EmojiZwj => "👨‍👩‍👧‍👦",
            Self::ArabicRtl => "العربية ",
            Self::MixedBidi => "abc אבג ",
        }
    }

    fn request<'a>(
        self,
        text: &'a str,
        style: &'a TextStyle,
        source_start: usize,
    ) -> BackendShapeRequest<'a> {
        let source_range = TextRange {
            start: source_start,
            end: source_start.saturating_add(text.len()),
        };
        match self {
            Self::VerticalCjk => BackendShapeRequest::vertical(
                text,
                style,
                TextDirection::LeftToRight,
                source_range,
                VerticalMode::Mixed,
            ),
            Self::ArabicRtl => BackendShapeRequest::horizontal(
                text,
                style,
                TextDirection::RightToLeft,
                source_range,
            ),
            Self::MixedBidi => {
                BackendShapeRequest::horizontal(text, style, TextDirection::Auto, source_range)
            }
            _ => BackendShapeRequest::horizontal(
                text,
                style,
                TextDirection::LeftToRight,
                source_range,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SpanMode {
    Single,
    Alternating,
}

impl SpanMode {
    const ALL: [Self; 2] = [Self::Single, Self::Alternating];

    const fn label(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Alternating => "alternating",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TrackingMode {
    Zero,
    Candidate,
}

impl TrackingMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::Candidate => "candidate_nonzero",
        }
    }
}

struct ProfileFixture {
    workload: Workload,
    span_mode: SpanMode,
    requested_cluster_count: usize,
    requested_bytes: usize,
    spans: Vec<ProfileSpan>,
}

impl ProfileFixture {
    fn new(workload: Workload, span_mode: SpanMode, cluster_count: usize) -> Self {
        let text = exact_grapheme_count(workload.seed(), cluster_count);
        let spans = match span_mode {
            SpanMode::Single => vec![ProfileSpan::new(text, 0, false)],
            SpanMode::Alternating => alternating_spans(&text),
        };
        let requested_bytes = spans.iter().map(|span| span.text.len()).sum();
        assert_eq!(
            spans
                .iter()
                .map(|span| span.text.graphemes(true).count())
                .sum::<usize>(),
            cluster_count
        );
        Self {
            workload,
            span_mode,
            requested_cluster_count: cluster_count,
            requested_bytes,
            spans,
        }
    }
}

struct ProfileSpan {
    text: String,
    source_start: usize,
    style: TextStyle,
}

impl ProfileSpan {
    fn new(text: String, source_start: usize, alternate: bool) -> Self {
        let mut style = test_style();
        style.font_size = 16.0;
        style.line_height = 20.0;
        style.font_weight = if alternate { 700 } else { 400 };
        Self {
            text,
            source_start,
            style,
        }
    }
}

fn exact_grapheme_count(seed: &str, cluster_count: usize) -> String {
    let seed_clusters = seed.graphemes(true).count();
    assert!(seed_clusters > 0);
    seed.repeat(cluster_count.div_ceil(seed_clusters))
        .graphemes(true)
        .take(cluster_count)
        .collect()
}

fn alternating_spans(text: &str) -> Vec<ProfileSpan> {
    let mut spans = Vec::new();
    let mut start = 0_usize;
    let mut clusters_in_span = 0_usize;
    for (offset, grapheme) in text.grapheme_indices(true) {
        clusters_in_span += 1;
        if clusters_in_span < ALTERNATING_SPAN_CLUSTERS {
            continue;
        }
        let end = offset + grapheme.len();
        spans.push(ProfileSpan::new(
            text[start..end].to_owned(),
            start,
            spans.len() % 2 == 1,
        ));
        start = end;
        clusters_in_span = 0;
    }
    if start < text.len() {
        spans.push(ProfileSpan::new(
            text[start..].to_owned(),
            start,
            spans.len() % 2 == 1,
        ));
    }
    spans
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ProfileResult {
    glyph_count: usize,
    cluster_count: usize,
    adjusted_gap_count: usize,
    bypassed_run_count: usize,
    glyph_wrap_line_count: usize,
    measured_width: f32,
    estimated_output_bytes: usize,
    routes: BackendRouteCounts,
}

impl ProfileResult {
    fn signature(self) -> (usize, usize, usize, usize, usize, u32, BackendRouteCounts) {
        (
            self.glyph_count,
            self.cluster_count,
            self.adjusted_gap_count,
            self.bypassed_run_count,
            self.glyph_wrap_line_count,
            self.measured_width.to_bits(),
            self.routes,
        )
    }
}

fn profile_fixture(fixture: &ProfileFixture, tracking_mode: TrackingMode) -> ProfileResult {
    let mut result = ProfileResult::default();
    for span in &fixture.spans {
        let mut features = span.style.features.to_vec();
        if matches!(tracking_mode, TrackingMode::Candidate) {
            features.push(OpenTypeFeature::new(*b"liga", 0));
        }
        let request = fixture
            .workload
            .request(&span.text, &span.style, span.source_start)
            .with_features(&features);
        let mut run = shape_text(request).expect("profile input must resolve a rasterizable face");
        result.routes.record(&run);
        if matches!(tracking_mode, TrackingMode::Candidate) {
            let applied = apply_candidate_tracking(&mut run, TRACKING_PX);
            result.adjusted_gap_count = result
                .adjusted_gap_count
                .saturating_add(applied.adjusted_gap_count);
            result.bypassed_run_count = result
                .bypassed_run_count
                .saturating_add(applied.bypassed as usize);
        }
        result.glyph_count = result.glyph_count.saturating_add(run_glyph_count(&run));
        result.cluster_count = result.cluster_count.saturating_add(run_cluster_count(&run));
        result.glyph_wrap_line_count = result
            .glyph_wrap_line_count
            .saturating_add(glyph_wrap_line_count(&run, GLYPH_WRAP_WIDTH));
        result.measured_width += run.measured_width;
    }
    result.estimated_output_bytes = result.glyph_count.saturating_mul(size_of::<ShapedGlyph>());
    result
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TrackingApplication {
    adjusted_gap_count: usize,
    bypassed: bool,
}

fn apply_candidate_tracking(run: &mut ShapedGlyphRun, tracking: f32) -> TrackingApplication {
    let supported = tracking.is_finite()
        && tracking >= 0.0
        && run.orientation == TextOrientation::Horizontal
        && run.direction == TextDirection::LeftToRight
        && run.lines.iter().all(|line| {
            line.glyphs.iter().all(|glyph| {
                glyph.direction == TextDirection::LeftToRight && !glyph.cluster_flags.rtl
            })
        });
    if !supported {
        return TrackingApplication {
            bypassed: true,
            ..TrackingApplication::default()
        };
    }

    let mut adjusted_gap_count = 0_usize;
    for line in &mut run.lines {
        let mut accumulated_shift = 0.0_f32;
        for glyph_index in 0..line.glyphs.len() {
            line.glyphs[glyph_index].x += accumulated_shift;
            let has_next_cluster = line
                .glyphs
                .get(glyph_index + 1)
                .is_some_and(|glyph| glyph.cluster_flags.cluster_start);
            if has_next_cluster && !line.glyphs[glyph_index].cluster_flags.virtual_glyph {
                line.glyphs[glyph_index].advance += tracking;
                accumulated_shift += tracking;
                adjusted_gap_count = adjusted_gap_count.saturating_add(1);
            }
        }
        line.measured_width += accumulated_shift;
    }
    run.measured_width = run
        .lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    TrackingApplication {
        adjusted_gap_count,
        bypassed: false,
    }
}

fn glyph_wrap_line_count(run: &ShapedGlyphRun, max_width: f32) -> usize {
    run.lines
        .iter()
        .map(|line| {
            let mut visual_lines = usize::from(!line.glyphs.is_empty());
            let mut width = 0.0_f32;
            for glyph in &line.glyphs {
                let advance = glyph.advance.max(0.0);
                if width > 0.0 && width + advance > max_width {
                    visual_lines = visual_lines.saturating_add(1);
                    width = 0.0;
                }
                width += advance;
            }
            visual_lines
        })
        .sum()
}

fn run_glyph_count(run: &ShapedGlyphRun) -> usize {
    run.lines.iter().map(|line| line.glyphs.len()).sum()
}

fn run_cluster_count(run: &ShapedGlyphRun) -> usize {
    run.lines
        .iter()
        .flat_map(|line| &line.glyphs)
        .filter(|glyph| glyph.cluster_flags.cluster_start)
        .count()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BackendRouteCounts {
    direct: usize,
    alternate: usize,
    hybrid: usize,
}

impl BackendRouteCounts {
    fn record(&mut self, run: &ShapedGlyphRun) {
        match run.horizontal_composition_receipt.as_deref() {
            None => self.direct = self.direct.saturating_add(1),
            Some(receipt) if receipt.alternate_ranges.is_empty() => {
                self.alternate = self.alternate.saturating_add(1);
            }
            Some(_) => self.hybrid = self.hybrid.saturating_add(1),
        }
    }

    fn label(self) -> String {
        format!(
            "direct:{}|alternate:{}|hybrid:{}",
            self.direct, self.alternate, self.hybrid
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct BackendWork {
    backend_shape_calls: usize,
    shaping_request_count: usize,
}

fn capture_backend_work(fixture: &ProfileFixture, tracking_mode: TrackingMode) -> BackendWork {
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = format!(
        "text07-letter-spacing-{}-{}-{}",
        fixture.workload.label(),
        fixture.span_mode.label(),
        tracking_mode.label()
    );
    config.max_counters = fixture.spans.len().saturating_mul(32).saturating_add(64);
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);
    black_box(profile_fixture(fixture, tracking_mode));
    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(!crate::core::diagnostics::profiling::reset_capture().active);

    BackendWork {
        backend_shape_calls: profile_counter_total(
            &snapshot,
            "text_direct_backend_shape_call_count",
        )
        .saturating_add(profile_counter_total(
            &snapshot,
            "text_horizontal_hybrid_direct_backend_shape_call_count",
        )),
        shaping_request_count: profile_counter_total(&snapshot, "text_analysis_request_count"),
    }
}

fn profile_counter_total(
    snapshot: &crate::core::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> usize {
    snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value as usize)
        .sum()
}

#[derive(Clone, Copy, Debug)]
struct CacheWork {
    cold: ShapedRunCacheReport,
    warm: ShapedRunCacheReport,
}

fn capture_current_cache_baseline(fixture: &ProfileFixture) -> CacheWork {
    let mut cache = ShapedRunCache::new();
    cache.begin_frame(1);
    for span in &fixture.spans {
        let raw_request = fixture
            .workload
            .request(&span.text, &span.style, span.source_start);
        let canonical = raw_request
            .canonicalized()
            .expect("profile cache request must canonicalize");
        let request = canonical.request();
        let key = ShapedRunCacheKey::from_request(&request);
        assert!(cache.get(&key, &span.text).is_none());
        let run = shape_text(request).expect("cold cache request must shape");
        cache.insert(key, run);
    }
    let cold = cache.report();

    cache.begin_frame(2);
    for span in &fixture.spans {
        let raw_request = fixture
            .workload
            .request(&span.text, &span.style, span.source_start);
        let canonical = raw_request
            .canonicalized()
            .expect("profile cache request must canonicalize");
        let request = canonical.request();
        let key = ShapedRunCacheKey::from_request(&request);
        assert!(cache.get(&key, &span.text).is_some());
    }
    CacheWork {
        cold,
        warm: cache.report(),
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = percentile.saturating_mul(sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

#[cfg(windows)]
mod rss {
    use std::ffi::c_void;
    use std::mem::{size_of, MaybeUninit};

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

    pub(super) fn current_rss_bytes() -> usize {
        let mut counters = MaybeUninit::<ProcessMemoryCounters>::zeroed();
        let counters_ptr = counters.as_mut_ptr();
        // SAFETY: the ABI-sized counter buffer and process handle remain valid for this OS call.
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
}

#[cfg(windows)]
use rss::current_rss_bytes;

#[cfg(not(windows))]
fn current_rss_bytes() -> usize {
    0
}
