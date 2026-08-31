#![cfg(feature = "profiling")]

use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextWrap},
};

use crate::{text::SharedTextLayoutSession, ui::text::UiTextViewport};

use super::super::{layout_profile_metrics_enabled, layout_text_with_provider_and_viewport};
use super::{layout_text, measure_text_size, test_style};

const PROFILE_SAMPLE_COUNT: usize = 31;
const BLOCK_PARAGRAPH_PROFILE_COUNTS: [usize; 4] = [1, 100, 1_000, 10_000];
const BLOCK_PARAGRAPH_PROFILE_FRAME: UiFrame = UiFrame::new(0.0, 0.0, 96.0, 1_000_000.0);

#[test]
fn arabic_justify_reports_one_aggregate_tatweel_receipt_per_candidate_line() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "arabic-tatweel-line-receipt".to_owned();
    config.max_spans = 256;
    config.max_counters = 256;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_align = UiTextAlign::Justify;
    style.text_direction = UiTextDirection::RightToLeft;
    let target_width = measure_text_size("سلام", &style).width + 18.0;
    let layout = layout_text(
        "سلام\nذ",
        &style,
        UiFrame::new(0.0, 0.0, target_width, 24.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(!crate::core::diagnostics::profiling::reset_capture().active);
    assert!(layout.lines[0].text.contains('\u{0640}'));
    assert_span_count(&snapshot, "arabic_tatweel_candidate_fit", 1);

    let requested_count = counter_value(&snapshot, "arabic_tatweel_requested_count");
    let probe_count = counter_value(&snapshot, "arabic_tatweel_probe_count");
    let candidate_bytes = counter_value(&snapshot, "arabic_tatweel_candidate_input_byte_count");
    let safe_candidate_count = counter_value(&snapshot, "arabic_tatweel_safe_candidate_count");
    let accepted_count = counter_value(&snapshot, "arabic_tatweel_accepted_count");
    let rejection_code = counter_value(&snapshot, "arabic_tatweel_last_rejection_code");
    let max_tatweels = counter_value(&snapshot, "text.runtime_budget.arabic_tatweels_per_line");
    let max_fit_measurements = counter_value(
        &snapshot,
        "text.runtime_budget.arabic_tatweel_fit_measurements_per_line",
    );

    assert!((1.0..=32.0).contains(&requested_count));
    assert!((1.0..=5.0).contains(&probe_count));
    assert!(candidate_bytes >= "سلامـ".len() as f64);
    assert!((1.0..=probe_count).contains(&safe_candidate_count));
    assert!((1.0..=32.0).contains(&accepted_count));
    assert!((0.0..=14.0).contains(&rejection_code));
    assert_eq!(max_tatweels, 32.0);
    assert_eq!(max_fit_measurements, 5.0);
}

#[test]
fn plain_layout_reports_final_line_fragment_cache_deltas() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "plain-layout-pre-artifact-cache-deltas".to_owned();
    config.max_spans = 64;
    config.max_counters = 32;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let layout = layout_text(
        "Measured wrapping keeps layout and artifact telemetry distinct.",
        &test_style(UiTextWrap::Word, UiTextOverflow::Clip),
        UiFrame::new(0.0, 0.0, 55.0, 120.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert!(
        layout.lines.len() > 1,
        "sample must exercise wrapped plain text"
    );
    for (category, name) in [
        ("text.layout", "resolve_without_artifact"),
        ("text.artifact", "build_resolved_text_glyph_artifact"),
    ] {
        assert_eq!(
            snapshot
                .spans
                .iter()
                .filter(|span| span.category == category && span.name == name)
                .count(),
            1,
            "plain layout must expose exactly one {category}/{name} span"
        );
    }
    for name in [
        "layout_pre_artifact_shaped_cache_hit_count",
        "layout_pre_artifact_shaped_cache_miss_count",
    ] {
        let value = snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value)
            .expect("layout must report its pre-artifact cache delta");
        assert!(
            value.is_finite() && value >= 0.0,
            "layout must report a finite non-negative {name}"
        );
    }
    let physical_line_request_count = snapshot
        .counters
        .iter()
        .find(|counter| {
            counter.stream == "runtime"
                && counter.name == "physical_line_fragment_initial_shape_request_count"
        })
        .map(|counter| counter.value)
        .expect("plain layout must report final physical-line fragment requests");
    assert!(
        physical_line_request_count.is_finite()
            && physical_line_request_count >= layout.lines.len() as f64,
        "each published non-empty physical line must originate from an initial fragment"
    );
    for name in [
        "physical_line_fragment_shaped_cache_hit_count",
        "physical_line_fragment_shaped_cache_miss_count",
    ] {
        let value = snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value)
            .expect("plain layout must report its physical-line cache delta");
        assert!(
            value.is_finite() && value >= 0.0,
            "layout must report a finite non-negative {name}"
        );
    }
    for (name, expected) in [
        (
            "artifact_build_retained_fragment_projection_count",
            layout.lines.len() as f64,
        ),
        ("artifact_build_fallback_shape_request_count", 0.0),
    ] {
        assert_eq!(
            snapshot
                .counters
                .iter()
                .find(|counter| counter.stream == "runtime" && counter.name == name)
                .map(|counter| counter.value),
            Some(expected),
            "plain source-congruent layout must use the retained artifact path for {name}"
        );
    }
}

#[test]
fn virtual_ellipsis_projects_the_retained_logical_fragment() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "virtual-ellipsis-retained-logical-fragment".to_owned();
    config.max_spans = 64;
    config.max_counters = 32;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let style = test_style(UiTextWrap::None, UiTextOverflow::Ellipsis);
    let frame_width = measure_text_size("ab…", &style).width + 0.1;
    let layout = layout_text(
        "abcdef",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 24.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert_eq!(
        layout.lines.len(),
        1,
        "fixture must produce one virtual line"
    );
    assert!(
        layout.lines[0].ellipsized,
        "fixture must generate an ellipsis run"
    );
    assert_eq!(layout.lines[0].text, "ab…");
    assert!(
        layout.rich_text_artifact.is_some(),
        "source-congruent virtual output must publish its glyph artifact"
    );
    for (name, expected_value) in [
        ("logical_virtual_fragment_shape_request_count", 1),
        (
            "artifact_build_retained_logical_virtual_fragment_projection_count",
            1,
        ),
        (
            "artifact_build_logical_virtual_projection_shape_request_count",
            0,
        ),
        ("artifact_build_fallback_shape_request_count", 0),
    ] {
        assert_counter_value(&snapshot, name, expected_value);
    }
}

#[test]
fn soft_hyphen_projects_the_retained_logical_fragment_without_renderer_reshape() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("pre-", &style).width + 0.1;
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "soft-hyphen-retained-logical-fragment".to_owned();
    config.max_spans = 64;
    config.max_counters = 32;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let layout = layout_text(
        "pre\u{00ad}fix",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert_eq!(layout.lines.len(), 2, "fixture must wrap at U+00AD");
    assert_eq!(layout.lines[0].text, "pre-");
    assert!(
        layout.rich_text_artifact.is_some(),
        "plain soft-hyphen output must publish its retained glyph artifact"
    );
    for (name, expected_value) in [
        ("logical_virtual_fragment_shape_request_count", 1),
        (
            "artifact_build_retained_logical_virtual_fragment_projection_count",
            1,
        ),
        (
            "artifact_build_logical_virtual_projection_shape_request_count",
            0,
        ),
        ("artifact_build_fallback_shape_request_count", 0),
    ] {
        assert_counter_value(&snapshot, name, expected_value);
    }
}

#[test]
fn block_layout_reports_paragraph_inset_resolution_work() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "block-layout-paragraph-inset-resolution".to_owned();
    config.max_spans = 64;
    config.max_counters = 16;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[ul][li]alpha beta gamma delta[/li][/ul]",
        &style,
        UiFrame::new(0.0, 0.0, 74.0, 160.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert!(
        layout.lines.len() >= 2,
        "fixture must exercise list wrapping"
    );
    assert_span_count(&snapshot, "resolve_paragraph_insets", 0);
    assert_span_count(&snapshot, "materialize_full_document_lines", 1);
    assert_span_count(&snapshot, "resolve_paragraph_line_constraints", 1);
    assert_counter_value(&snapshot, "paragraph_wrap_inset_resolution_count", 1);
    assert_counter_value(&snapshot, "paragraph_constraint_inset_resolution_count", 2);
}

#[test]
fn rich_glyph_layout_reports_aggregate_shape_work_by_phase() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "rich-glyph-layout-shape-work-by-phase".to_owned();
    config.max_spans = 64;
    config.max_counters = 256;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[b]alpha[/b] [size=24]beta gamma[/size]",
        &style,
        UiFrame::new(0.0, 0.0, 48.0, 160.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert!(
        layout.lines.len() >= 2,
        "fixture must exercise rich glyph wrapping"
    );
    for phase in [
        "rich_range_index",
        "rich_layout_materialization",
        "ui_rich_item_projection",
    ] {
        assert_span_count(&snapshot, phase, 1);
    }
    for counter in [
        "rich_range_index_shape_request_count",
        "rich_range_index_shape_input_byte_count",
        "rich_layout_shape_request_count",
        "rich_layout_shape_input_byte_count",
        "ui_rich_item_projection_shape_request_count",
        "ui_rich_item_projection_shape_input_byte_count",
    ] {
        let samples = snapshot
            .counters
            .iter()
            .filter(|sample| sample.stream == "runtime" && sample.name == counter)
            .collect::<Vec<_>>();
        assert_eq!(
            samples.len(),
            1,
            "{counter} must be emitted once per rich layout, not once per run"
        );
        assert!(
            samples[0].value.is_finite() && samples[0].value > 0.0,
            "{counter} must report finite positive work for the styled fixture"
        );
    }
}

#[test]
fn plain_viewport_reports_visible_line_selection_without_full_materialization() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "plain-viewport-visible-line-selection".to_owned();
    config.max_spans = 64;
    config.max_counters = 16;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let mut provider = SharedTextLayoutSession::new();
    let layout = layout_text_with_provider_and_viewport(
        "zero\none\ntwo\nthree",
        &test_style(UiTextWrap::None, UiTextOverflow::Clip),
        UiFrame::new(0.0, 0.0, 96.0, 120.0),
        None,
        UiTextViewport::new(13.0, 1.0, 0).expect("finite document viewport"),
        None,
        &mut provider,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "one");
    assert_span_count(&snapshot, "certify_plain_viewport_line_height", 1);
    assert_span_count(&snapshot, "select_visible_plain_lines", 1);
    assert_span_count(&snapshot, "materialize_full_document_lines", 0);
}

#[test]
fn block_viewport_reports_full_document_materialization_without_plain_selection() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "block-viewport-full-materialization".to_owned();
    config.max_spans = 64;
    config.max_counters = 16;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let mut provider = SharedTextLayoutSession::new();
    let layout = layout_text_with_provider_and_viewport(
        "[ul][li]alpha beta gamma delta[/li][/ul]",
        &style,
        UiFrame::new(0.0, 0.0, 74.0, 160.0),
        None,
        UiTextViewport::new(12.0, 12.0, 0).expect("finite document viewport"),
        None,
        &mut provider,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert!(layout.lines.len() >= 2);
    assert_span_count(&snapshot, "select_visible_plain_lines", 0);
    assert_span_count(&snapshot, "materialize_full_document_lines", 1);
}

#[test]
#[ignore = "manual 31-sample Text03 paragraph inset profiler trace; no machine-time acceptance threshold"]
fn block_layout_paragraph_inset_profile_reports_p50_p95() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;

    for paragraph_count in BLOCK_PARAGRAPH_PROFILE_COUNTS {
        let source = block_list_source(paragraph_count);
        let mut layout_samples_us = Vec::with_capacity(PROFILE_SAMPLE_COUNT);
        let mut full_materialization_samples_us = Vec::with_capacity(PROFILE_SAMPLE_COUNT);
        let mut paragraph_constraint_samples_us = Vec::with_capacity(PROFILE_SAMPLE_COUNT);
        let mut full_materialization_share_samples_basis_points =
            Vec::with_capacity(PROFILE_SAMPLE_COUNT);
        let mut paragraph_constraint_share_samples_basis_points =
            Vec::with_capacity(PROFILE_SAMPLE_COUNT);
        let mut line_count = 0_usize;

        for _ in 0..PROFILE_SAMPLE_COUNT {
            let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
            config.session_id = "block-layout-paragraph-inset-scale".to_owned();
            config.max_spans = 128;
            config.max_counters = 16;
            assert!(crate::core::diagnostics::profiling::start_capture(config).active);

            let layout = layout_text(&source, &style, BLOCK_PARAGRAPH_PROFILE_FRAME, None);
            let snapshot = crate::core::diagnostics::profiling::snapshot();
            assert!(
                !crate::core::diagnostics::profiling::reset_capture().active,
                "layout profiling capture must reset before another sample starts"
            );
            assert!(
                layout.lines.len() >= paragraph_count,
                "{paragraph_count} list items must materialize at least one line per physical paragraph"
            );

            assert_span_count(&snapshot, "resolve_paragraph_insets", 0);
            assert_counter_value(
                &snapshot,
                "paragraph_wrap_inset_resolution_count",
                paragraph_count,
            );
            assert_counter_value(
                &snapshot,
                "paragraph_constraint_inset_resolution_count",
                paragraph_count.saturating_mul(2),
            );
            let layout_spans = snapshot
                .spans
                .iter()
                .filter(|span| {
                    span.category == "text.layout" && span.name == "resolve_without_artifact"
                })
                .collect::<Vec<_>>();
            assert_eq!(
                layout_spans.len(),
                1,
                "one layout sample must retain exactly one root layout span"
            );
            let full_materialization_spans = snapshot
                .spans
                .iter()
                .filter(|span| {
                    span.category == "text.layout" && span.name == "materialize_full_document_lines"
                })
                .collect::<Vec<_>>();
            assert_eq!(
                full_materialization_spans.len(),
                1,
                "block layout must retain exactly one full-document materialization span"
            );
            let paragraph_constraint_spans = snapshot
                .spans
                .iter()
                .filter(|span| {
                    span.category == "text.layout"
                        && span.name == "resolve_paragraph_line_constraints"
                })
                .collect::<Vec<_>>();
            assert_eq!(
                paragraph_constraint_spans.len(),
                1,
                "block layout must retain exactly one paragraph-constraint span"
            );

            let full_materialization_duration_us = full_materialization_spans[0].duration_us;
            let paragraph_constraint_duration_us = paragraph_constraint_spans[0].duration_us;
            let layout_duration_us = layout_spans[0].duration_us;
            let accounted_duration_us =
                full_materialization_duration_us.saturating_add(paragraph_constraint_duration_us);
            assert!(
                accounted_duration_us <= layout_duration_us,
                "full-document wrapping and paragraph constraints must stay within the root layout: full={full_materialization_duration_us}us constraints={paragraph_constraint_duration_us}us layout={layout_duration_us}us"
            );
            layout_samples_us.push(layout_duration_us);
            full_materialization_samples_us.push(full_materialization_duration_us);
            paragraph_constraint_samples_us.push(paragraph_constraint_duration_us);
            full_materialization_share_samples_basis_points.push(
                full_materialization_duration_us.saturating_mul(10_000) / layout_duration_us.max(1),
            );
            paragraph_constraint_share_samples_basis_points.push(
                paragraph_constraint_duration_us.saturating_mul(10_000) / layout_duration_us.max(1),
            );
            line_count = layout.lines.len();
        }

        let (layout_p50_us, layout_p95_us) = p50_p95(&mut layout_samples_us);
        let (full_materialization_p50_us, full_materialization_p95_us) =
            p50_p95(&mut full_materialization_samples_us);
        let (paragraph_constraint_p50_us, paragraph_constraint_p95_us) =
            p50_p95(&mut paragraph_constraint_samples_us);
        let (
            full_materialization_share_p50_basis_points,
            full_materialization_share_p95_basis_points,
        ) = p50_p95(&mut full_materialization_share_samples_basis_points);
        let (
            paragraph_constraint_share_p50_basis_points,
            paragraph_constraint_share_p95_basis_points,
        ) = p50_p95(&mut paragraph_constraint_share_samples_basis_points);
        println!(
            "text03_block_paragraph_inset_profile paragraphs={paragraph_count} wrap=word_smart \\
             frame_width=96 lines={line_count} expected_wrap_inset_resolutions={paragraph_count} \\
             expected_constraint_inset_resolutions={} \\
             layout_p50_us={layout_p50_us} layout_p95_us={layout_p95_us} \\
             full_materialization_p50_us={full_materialization_p50_us} \\
             full_materialization_p95_us={full_materialization_p95_us} \\
             paragraph_constraint_p50_us={paragraph_constraint_p50_us} \\
             paragraph_constraint_p95_us={paragraph_constraint_p95_us} \\
             full_materialization_share_p50_basis_points={full_materialization_share_p50_basis_points} \\
             full_materialization_share_p95_basis_points={full_materialization_share_p95_basis_points} \\
             paragraph_constraint_share_p50_basis_points={paragraph_constraint_share_p50_basis_points} \\
             paragraph_constraint_share_p95_basis_points={paragraph_constraint_share_p95_basis_points}",
            paragraph_count.saturating_mul(2),
        );
    }
}

#[cfg(not(feature = "profiling-tracy"))]
#[test]
fn plain_layout_idle_cpu_profiler_skips_cache_measurement() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    assert!(
        !crate::core::diagnostics::profiling::capture_active(),
        "the idle-profiler contract requires capture to be inactive"
    );
    assert!(
        !layout_profile_metrics_enabled(),
        "idle CPU profiling must not read cache reports around every layout request"
    );
}

fn block_list_source(paragraph_count: usize) -> String {
    const LIST_ITEM: &str = "[li]alpha beta gamma delta epsilon zeta[/li]";

    let mut source = String::with_capacity(
        LIST_ITEM
            .len()
            .saturating_mul(paragraph_count)
            .saturating_add("[ul][/ul]".len()),
    );
    source.push_str("[ul]");
    for _ in 0..paragraph_count {
        source.push_str(LIST_ITEM);
    }
    source.push_str("[/ul]");
    source
}

fn p50_p95(samples_us: &mut [u64]) -> (u64, u64) {
    samples_us.sort_unstable();
    let p50_us = samples_us[samples_us.len() / 2];
    let p95_index = (samples_us.len() * 95).div_ceil(100) - 1;
    (p50_us, samples_us[p95_index])
}

fn assert_span_count(
    snapshot: &crate::core::diagnostics::profiling::ProfileSnapshot,
    name: &str,
    expected_count: usize,
) {
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| span.category == "text.layout" && span.name == name)
            .count(),
        expected_count,
        "expected {expected_count} text.layout/{name} spans"
    );
}

fn assert_counter_value(
    snapshot: &crate::core::diagnostics::profiling::ProfileSnapshot,
    name: &str,
    expected_value: usize,
) {
    let counters = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_eq!(
        counters.len(),
        1,
        "expected exactly one runtime/{name} aggregate counter"
    );
    assert_eq!(
        counters[0].value, expected_value as f64,
        "runtime/{name} must report the expected aggregate count"
    );
}

fn counter_value(
    snapshot: &crate::core::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> f64 {
    let counters = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_eq!(
        counters.len(),
        1,
        "expected exactly one runtime/{name} aggregate counter"
    );
    counters[0].value
}
