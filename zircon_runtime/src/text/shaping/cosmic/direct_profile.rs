use std::cell::Cell;

use crate::text::ShapedGlyphRun;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DirectShapeProfileMetrics {
    backend_shape_calls: usize,
    vertical_substitution_compare_calls: usize,
    vertical_substitution_compare_input_bytes: usize,
    vertical_substitution_compare_output_glyphs: usize,
    vertical_substitution_changed_clusters: usize,
}

thread_local! {
    static DIRECT_SHAPE_PROFILE_METRICS: Cell<Option<DirectShapeProfileMetrics>> = const { Cell::new(None) };
}

/// Starts request-local aggregation only while a managed capture is active. Backend leaves update
/// integers in TLS so segment count never becomes profiler lock count.
pub(super) fn begin() {
    DIRECT_SHAPE_PROFILE_METRICS.with(|metrics| {
        metrics.set(profile_metrics_enabled().then_some(DirectShapeProfileMetrics::default()));
    });
}

pub(super) fn discard() {
    DIRECT_SHAPE_PROFILE_METRICS.with(|metrics| metrics.set(None));
}

/// Detaches request-local direct metrics before an alternate backend is entered. The caller owns
/// the detached value until composition succeeds or fails, so an early Cosmic error cannot leak
/// pending state into the next request on the same thread.
pub(super) fn detach() -> Option<DirectShapeProfileMetrics> {
    take()
}

/// Publishes only a completed direct request. Fallback shaping deliberately does not contribute to
/// this stream, so the scale harness can reject a regression to a second backend path.
pub(super) fn record_completed_request(shaped: &ShapedGlyphRun, text: &str) {
    let Some(metrics) = take() else {
        return;
    };
    let glyph_count = shaped
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();
    crate::profile_counter!("runtime", "text_direct_shape_request_count", 1);
    crate::profile_counter!("runtime", "text_direct_shape_input_byte_count", text.len());
    crate::profile_counter!(
        "runtime",
        "text_direct_shape_output_glyph_count",
        glyph_count
    );
    crate::profile_counter!(
        "runtime",
        "text_direct_backend_shape_call_count",
        metrics.backend_shape_calls
    );
    crate::profile_counter!(
        "runtime",
        "text_direct_vertical_substitution_compare_call_count",
        metrics.vertical_substitution_compare_calls
    );
    crate::profile_counter!(
        "runtime",
        "text_direct_vertical_substitution_compare_input_byte_count",
        metrics.vertical_substitution_compare_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        "text_direct_vertical_substitution_compare_output_glyph_count",
        metrics.vertical_substitution_compare_output_glyphs
    );
    crate::profile_counter!(
        "runtime",
        "text_direct_vertical_substitution_changed_cluster_count",
        metrics.vertical_substitution_changed_clusters
    );
}

pub(super) fn record_horizontal_composition(
    metrics: Option<DirectShapeProfileMetrics>,
    input_bytes: usize,
    hole_count: usize,
    direct_glyph_count: usize,
    alternate_glyph_count: usize,
    rejected: bool,
) {
    let Some(metrics) = metrics else {
        return;
    };
    crate::profile_counter!(
        "runtime",
        "text_horizontal_hybrid_candidate_request_count",
        1
    );
    crate::profile_counter!(
        "runtime",
        "text_horizontal_hybrid_input_byte_count",
        input_bytes
    );
    crate::profile_counter!("runtime", "text_horizontal_hybrid_hole_count", hole_count);
    crate::profile_counter!(
        "runtime",
        "text_horizontal_hybrid_retained_direct_glyph_count",
        direct_glyph_count
    );
    crate::profile_counter!(
        "runtime",
        "text_horizontal_hybrid_selected_alternate_glyph_count",
        alternate_glyph_count
    );
    crate::profile_counter!(
        "runtime",
        "text_horizontal_hybrid_rejected_composition_count",
        rejected as usize
    );
    crate::profile_counter!(
        "runtime",
        "text_horizontal_hybrid_direct_backend_shape_call_count",
        metrics.backend_shape_calls
    );
}

pub(in crate::text::shaping) fn record_backend_shape_call() {
    update(|metrics| {
        metrics.backend_shape_calls = metrics.backend_shape_calls.saturating_add(1);
    });
}

pub(in crate::text::shaping) fn record_vertical_substitution_comparison(
    input_bytes: usize,
    output_glyphs: usize,
    changed_clusters: usize,
) {
    update(|metrics| {
        metrics.vertical_substitution_compare_calls = metrics
            .vertical_substitution_compare_calls
            .saturating_add(1);
        metrics.vertical_substitution_compare_input_bytes = metrics
            .vertical_substitution_compare_input_bytes
            .saturating_add(input_bytes);
        metrics.vertical_substitution_compare_output_glyphs = metrics
            .vertical_substitution_compare_output_glyphs
            .saturating_add(output_glyphs);
        metrics.vertical_substitution_changed_clusters = metrics
            .vertical_substitution_changed_clusters
            .saturating_add(changed_clusters);
    });
}

fn update(update: impl FnOnce(&mut DirectShapeProfileMetrics)) {
    DIRECT_SHAPE_PROFILE_METRICS.with(|metrics| {
        let Some(mut current) = metrics.get() else {
            return;
        };
        update(&mut current);
        metrics.set(Some(current));
    });
}

fn take() -> Option<DirectShapeProfileMetrics> {
    DIRECT_SHAPE_PROFILE_METRICS.with(|metrics| metrics.replace(None))
}

fn profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}
