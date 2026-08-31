use std::time::Instant;

use crate::core::framework::text::TextDirection;
use crate::text::{BackendShapeRequest, TextRange, TextShapingWorkBudget, TextStyle, VerticalMode};
#[cfg(feature = "profiling")]
use unicode_segmentation::UnicodeSegmentation;

use super::super::shape_text;
use super::test_style;

#[test]
#[ignore = "managed Text02 capture-inactive direct-shaping 31-sample timing evidence; no machine-time acceptance threshold"]
fn direct_shaping_scale_evidence_reports_capture_inactive_p50_p95() {
    const SAMPLE_COUNT: usize = 31;
    const SCALE_UNITS: [usize; 4] = [1, 100, 1_000, 10_000];

    let style = test_style();
    for workload in DirectShapeScaleWorkload::ALL {
        for unit_count in SCALE_UNITS {
            let text = workload.unit().repeat(unit_count);
            let mut samples_ns = Vec::with_capacity(SAMPLE_COUNT);
            for _ in 0..SAMPLE_COUNT {
                let started = Instant::now();
                let shaped = shape_text(workload.request(&text, &style))
                    .expect("scale workload must resolve a rasterizable face");
                samples_ns.push(started.elapsed().as_nanos());
                assert!(
                    shaped.lines.iter().any(|line| !line.glyphs.is_empty()),
                    "{} at {unit_count} units must produce direct backend glyphs",
                    workload.label()
                );
            }

            println!(
                "TEXT02_DIRECT_SHAPING_TIME workload={} units={unit_count} samples={SAMPLE_COUNT} capture=inactive p50_ns={} p95_ns={}",
                workload.label(),
                percentile_ns(&mut samples_ns, 50),
                percentile_ns(&mut samples_ns, 95),
            );
        }
    }
}

#[test]
#[ignore = "managed Text02 long-line semantic-request timing evidence; no machine-time acceptance threshold"]
fn direct_shaping_long_semantic_request_evidence_reports_p50_p95() {
    const SAMPLE_COUNT: usize = 31;

    let style = test_style();
    let budget = TextShapingWorkBudget::default();
    for workload in DirectShapeScaleWorkload::ALL {
        let unit_count =
            (budget.max_inline_input_bytes() / workload.unit().len()).saturating_add(17);
        let text = workload.unit().repeat(unit_count);
        assert!(budget.exceeds_inline_threshold(text.len()));
        let mut samples_ns = Vec::with_capacity(SAMPLE_COUNT);
        for _ in 0..SAMPLE_COUNT {
            let started = Instant::now();
            let shaped = shape_text(workload.request(&text, &style))
                .expect("long semantic workload must resolve a rasterizable face");
            samples_ns.push(started.elapsed().as_nanos());
            assert_eq!(
                shaped.lines.len(),
                1,
                "{} must retain one logical source line across the inline-work threshold",
                workload.label()
            );
            assert!(
                shaped.lines[0]
                    .glyphs
                    .iter()
                    .all(|glyph| !glyph.cluster_flags.virtual_glyph)
            );
            assert!(
                shaped.lines[0]
                    .glyphs
                    .iter()
                    .any(|glyph| glyph.font_id.is_some())
            );
        }

        println!(
            "TEXT02_LONG_SEMANTIC_REQUEST_TIME workload={} bytes={} samples={SAMPLE_COUNT} p50_ns={} p95_ns={}",
            workload.label(),
            text.len(),
            percentile_ns(&mut samples_ns, 50),
            percentile_ns(&mut samples_ns, 95),
        );
    }
}

#[cfg(feature = "profiling")]
#[test]
#[ignore = "managed Text02 direct-shaping and vertical-comparison counter topology evidence"]
fn direct_shaping_counter_evidence_reports_aggregated_backend_calls() {
    const SAMPLE_COUNT: usize = 31;
    const SCALE_UNITS: [usize; 4] = [1, 100, 1_000, 10_000];
    const DIRECT_SHAPE_COUNTERS_PER_SAMPLE: usize = 8;

    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let style = test_style();
    for workload in DirectShapeScaleWorkload::ALL {
        for unit_count in SCALE_UNITS {
            let text = workload.unit().repeat(unit_count);
            let expected_request_count = SAMPLE_COUNT as f64;
            let expected_input_bytes = text.len().saturating_mul(SAMPLE_COUNT) as f64;
            let maximum_segment_calls = text.graphemes(true).count().saturating_mul(SAMPLE_COUNT);
            let maximum_backend_calls = maximum_segment_calls.saturating_mul(2) as f64;
            let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
            config.session_id = format!("text02-direct-shaping-{}-{unit_count}", workload.label());
            config.max_counters = SAMPLE_COUNT * DIRECT_SHAPE_COUNTERS_PER_SAMPLE;
            assert!(crate::core::diagnostics::profiling::start_capture(config).active);

            for _ in 0..SAMPLE_COUNT {
                let shaped = shape_text(workload.request(&text, &style))
                    .expect("profile workload must resolve a rasterizable face");
                assert!(
                    shaped.lines.iter().any(|line| !line.glyphs.is_empty()),
                    "{} at {unit_count} units must produce direct backend glyphs",
                    workload.label()
                );
            }

            let snapshot = crate::core::diagnostics::profiling::snapshot();
            assert!(
                !crate::core::diagnostics::profiling::reset_capture().active,
                "direct shaping profiling capture must reset before the next workload"
            );
            assert_eq!(
                snapshot.counters.len(),
                SAMPLE_COUNT * DIRECT_SHAPE_COUNTERS_PER_SAMPLE,
                "each direct shape must publish a fixed low-cardinality request and vertical-comparison counter set"
            );
            let direct_requests =
                profile_counter_total(&snapshot, "text_direct_shape_request_count");
            let backend_calls =
                profile_counter_total(&snapshot, "text_direct_backend_shape_call_count");
            let direct_input_bytes =
                profile_counter_total(&snapshot, "text_direct_shape_input_byte_count");
            let direct_output_glyphs =
                profile_counter_total(&snapshot, "text_direct_shape_output_glyph_count");
            let comparison_calls = profile_counter_total(
                &snapshot,
                "text_direct_vertical_substitution_compare_call_count",
            );
            let comparison_input_bytes = profile_counter_total(
                &snapshot,
                "text_direct_vertical_substitution_compare_input_byte_count",
            );
            let comparison_output_glyphs = profile_counter_total(
                &snapshot,
                "text_direct_vertical_substitution_compare_output_glyph_count",
            );
            let changed_clusters = profile_counter_total(
                &snapshot,
                "text_direct_vertical_substitution_changed_cluster_count",
            );

            assert_eq!(direct_requests, expected_request_count);
            assert_eq!(direct_input_bytes, expected_input_bytes);
            assert!(
                backend_calls >= expected_request_count,
                "{} must make at least one real backend call per direct shape",
                workload.label()
            );
            assert!(
                backend_calls <= maximum_backend_calls,
                "{} backend calls must stay linear in grapheme count; calls={backend_calls}, upper={maximum_backend_calls}",
                workload.label()
            );
            assert!(direct_output_glyphs > 0.0);

            if workload.requires_vertical_substitution_comparison() {
                assert!(comparison_calls >= expected_request_count);
                assert!(comparison_calls <= maximum_segment_calls as f64);
                assert_eq!(comparison_input_bytes, expected_input_bytes);
                assert!(comparison_output_glyphs > 0.0);
                assert!(changed_clusters <= comparison_output_glyphs);
            } else {
                assert_eq!(comparison_calls, 0.0);
                assert_eq!(comparison_input_bytes, 0.0);
                assert_eq!(comparison_output_glyphs, 0.0);
                assert_eq!(changed_clusters, 0.0);
            }

            println!(
                "TEXT02_DIRECT_SHAPING_COUNTERS workload={} units={unit_count} samples={SAMPLE_COUNT} direct_requests={direct_requests} backend_calls={backend_calls} input_bytes={direct_input_bytes} output_glyphs={direct_output_glyphs} comparison_calls={comparison_calls} comparison_input_bytes={comparison_input_bytes} comparison_output_glyphs={comparison_output_glyphs} changed_clusters={changed_clusters}",
                workload.label(),
            );
        }
    }
}

#[derive(Clone, Copy)]
enum DirectShapeScaleWorkload {
    Latin,
    Cjk,
    Rtl,
    Ligature,
    VerticalCjk,
    VerticalTr,
}

impl DirectShapeScaleWorkload {
    const ALL: [Self; 6] = [
        Self::Latin,
        Self::Cjk,
        Self::Rtl,
        Self::Ligature,
        Self::VerticalCjk,
        Self::VerticalTr,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Latin => "latin",
            Self::Cjk => "cjk",
            Self::Rtl => "rtl",
            Self::Ligature => "ligature",
            Self::VerticalCjk => "vertical_cjk",
            Self::VerticalTr => "vertical_tr",
        }
    }

    const fn unit(self) -> &'static str {
        match self {
            Self::Latin => "A",
            Self::Cjk | Self::VerticalCjk => "汉",
            Self::Rtl => "ب",
            Self::Ligature => "office",
            Self::VerticalTr => "（",
        }
    }

    const fn requires_vertical_substitution_comparison(self) -> bool {
        matches!(self, Self::VerticalTr)
    }

    fn request<'a>(self, text: &'a str, style: &'a TextStyle) -> BackendShapeRequest<'a> {
        let source_range = TextRange {
            start: 0,
            end: text.len(),
        };
        match self {
            Self::VerticalCjk | Self::VerticalTr => BackendShapeRequest::vertical(
                text,
                style,
                TextDirection::LeftToRight,
                source_range,
                VerticalMode::Mixed,
            ),
            Self::Latin | Self::Cjk | Self::Rtl | Self::Ligature => {
                BackendShapeRequest::horizontal(
                    text,
                    style,
                    if matches!(self, Self::Rtl) {
                        TextDirection::RightToLeft
                    } else {
                        TextDirection::LeftToRight
                    },
                    source_range,
                )
            }
        }
    }
}

#[cfg(feature = "profiling")]
fn profile_counter_total(
    snapshot: &crate::core::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> f64 {
    snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value)
        .sum()
}

fn percentile_ns(samples: &mut [u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty(), "percentile requires samples");
    assert!((1..=100).contains(&percentile));
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[index]
}
