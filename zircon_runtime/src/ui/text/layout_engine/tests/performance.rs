use std::time::Instant;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextDirection, UiTextOverflow, UiTextWrap},
};

use crate::text::SharedTextLayoutSession;

use super::{layout_text_with_provider, test_style};

const SAMPLE_COUNT: usize = 31;
const SCALE_GRAPHEME_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];
const WRAP_FRAME: UiFrame = UiFrame::new(0.0, 0.0, 96.0, 1_000_000.0);

#[derive(Clone, Copy)]
struct LayoutScaleCase {
    name: &'static str,
    seed: &'static str,
    direction: UiTextDirection,
}

const LAYOUT_SCALE_CASES: [LayoutScaleCase; 4] = [
    LayoutScaleCase {
        name: "latin",
        seed: "office layout ",
        direction: UiTextDirection::LeftToRight,
    },
    LayoutScaleCase {
        name: "cjk",
        seed: "Zircon文本布局",
        direction: UiTextDirection::LeftToRight,
    },
    LayoutScaleCase {
        name: "rtl",
        seed: "مرحبا بالعالم ",
        direction: UiTextDirection::RightToLeft,
    },
    LayoutScaleCase {
        name: "ligature",
        seed: "ffi office ",
        direction: UiTextDirection::LeftToRight,
    },
];

#[derive(Clone, Copy, Default)]
struct CacheDelta {
    hit_count: u64,
    miss_count: u64,
}

#[test]
#[ignore = "manual 31-sample Text02 layout/artifact cold/warm evidence; no machine-time acceptance threshold"]
fn plain_layout_artifact_scale_reports_cold_and_warm_p50_p95() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    assert!(
        !crate::core::diagnostics::profiling::feature_enabled()
            && !cfg!(feature = "profiling-tracy"),
        "Text02 wall-clock evidence requires profiling and Tracy features to stay disabled"
    );

    for case in LAYOUT_SCALE_CASES {
        let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
        style.text_direction = case.direction;
        for grapheme_count in SCALE_GRAPHEME_COUNTS {
            let source = repeat_to_grapheme_count(case.seed, grapheme_count);
            let mut cold_samples_ns = Vec::with_capacity(SAMPLE_COUNT);
            let mut warm_samples_ns = Vec::with_capacity(SAMPLE_COUNT);
            let mut cold_cache_delta = CacheDelta::default();
            let mut warm_cache_delta = CacheDelta::default();
            let mut line_count = 0_usize;

            for _ in 0..SAMPLE_COUNT {
                let mut session = SharedTextLayoutSession::new();
                let cold_before = session.cache_report();
                let cold_started = Instant::now();
                let cold_layout =
                    layout_text_with_provider(&source, &style, WRAP_FRAME, None, &mut session);
                cold_samples_ns.push(cold_started.elapsed().as_nanos());
                let cold_after = session.cache_report();

                let warm_started = Instant::now();
                let warm_layout =
                    layout_text_with_provider(&source, &style, WRAP_FRAME, None, &mut session);
                warm_samples_ns.push(warm_started.elapsed().as_nanos());
                let warm_after = session.cache_report();

                assert!(
                    cold_layout.rich_text_artifact.is_some()
                        && warm_layout.rich_text_artifact.is_some(),
                    "{case_name} {grapheme_count} graphemes must exercise the glyph artifact path",
                    case_name = case.name,
                );
                assert_eq!(
                    cold_layout.lines.len(),
                    warm_layout.lines.len(),
                    "cold and warm requests must resolve the same line count"
                );
                assert_eq!(
                    cold_layout.measured_width.to_bits(),
                    warm_layout.measured_width.to_bits(),
                    "cold and warm requests must resolve the same width"
                );
                assert_eq!(
                    cold_layout.measured_height.to_bits(),
                    warm_layout.measured_height.to_bits(),
                    "cold and warm requests must resolve the same height"
                );

                cold_cache_delta = cache_delta(cold_before, cold_after);
                warm_cache_delta = cache_delta(cold_after, warm_after);
                line_count = cold_layout.lines.len();
            }

            let (cold_p50_ns, cold_p95_ns) = p50_p95(&mut cold_samples_ns);
            let (warm_p50_ns, warm_p95_ns) = p50_p95(&mut warm_samples_ns);
            println!(
                "text02_layout_artifact_scale case={} graphemes={grapheme_count} wrap=word \
                 frame_width=96 lines={line_count} cold_cache_hits={} cold_cache_misses={} \
                 warm_cache_hits={} warm_cache_misses={} cold_p50_ns={cold_p50_ns} \
                 cold_p95_ns={cold_p95_ns} warm_p50_ns={warm_p50_ns} warm_p95_ns={warm_p95_ns}",
                case.name,
                cold_cache_delta.hit_count,
                cold_cache_delta.miss_count,
                warm_cache_delta.hit_count,
                warm_cache_delta.miss_count,
            );
        }
    }
}

fn repeat_to_grapheme_count(seed: &str, grapheme_count: usize) -> String {
    let graphemes = seed.graphemes(true).collect::<Vec<_>>();
    assert!(
        !graphemes.is_empty(),
        "Text02 performance sample seed must contain at least one grapheme"
    );
    let mut source = String::with_capacity(seed.len().saturating_mul(grapheme_count));
    for index in 0..grapheme_count {
        source.push_str(graphemes[index % graphemes.len()]);
    }
    source
}

fn cache_delta(
    before: crate::text::cache::ShapedRunCacheReport,
    after: crate::text::cache::ShapedRunCacheReport,
) -> CacheDelta {
    CacheDelta {
        hit_count: after.hit_count.saturating_sub(before.hit_count),
        miss_count: after.miss_count.saturating_sub(before.miss_count),
    }
}

fn p50_p95(samples_ns: &mut [u128]) -> (u128, u128) {
    samples_ns.sort_unstable();
    let p50_ns = samples_ns[samples_ns.len() / 2];
    let p95_index = (samples_ns.len() * 95).div_ceil(100) - 1;
    (p50_ns, samples_ns[p95_index])
}
