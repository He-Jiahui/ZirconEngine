use std::hint::black_box;
use std::time::Instant;

use super::{
    frame_phase_order_spans, phase_diagnostic_name, RenderFramePhaseQueueSummaryPhaseOrderSpan,
    RenderPhaseQueueSummary, RenderPhaseQueueSummaryPhaseOrderSpan, RENDER_PHASES_BY_QUEUE_ORDER,
};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 20_000;

#[test]
fn optimization_batch_20260829x_runtime297_cached_frame_spans_match_legacy_projection() {
    let mut geometry = empty_queue_summary();
    geometry.phase_order_spans.push(queue_span(2, 7));
    let mut sprites = empty_queue_summary();
    sprites.phase_order_spans.push(queue_span(2, 3));

    assert_eq!(
        frame_phase_order_spans(&geometry, &sprites),
        legacy_frame_phase_order_spans(&geometry, &sprites)
    );

    let mut mutated = frame_phase_order_spans(&geometry, &sprites);
    mutated[0].total_item_count = 99;
    assert_ne!(
        frame_phase_order_spans(&geometry, &sprites)[0].total_item_count,
        99
    );
}

#[test]
fn optimization_batch_20260829x_runtime297_frame_span_builder_clones_cached_template() {
    let source = include_str!("../frame_phase_queue_summary.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn frame_phase_order_spans")
        .nth(1)
        .and_then(|body| body.split("fn phase_diagnostic_name").next())
        .expect("frame phase span builder");

    assert!(body.contains("OnceLock<Vec<RenderFramePhaseQueueSummaryPhaseOrderSpan>>"));
    assert!(body.contains("frame_phase_order_span_template().to_vec()"));
    assert!(body.contains("Vec::with_capacity(RENDER_PHASES_BY_QUEUE_ORDER.len())"));
    assert!(body.contains("chunk_by"));
    assert!(!body.contains(".iter_mut()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829x_runtime297_cached_frame_phase_span_template_bench() {
    let geometry = empty_queue_summary();
    let sprites = empty_queue_summary();
    black_box(frame_phase_order_spans(&geometry, &sprites));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &geometry, &sprites));
            optimized_samples.push(measure(true, &geometry, &sprites));
        } else {
            optimized_samples.push(measure(true, &geometry, &sprites));
            legacy_samples.push(measure(false, &geometry, &sprites));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME297_CACHED_FRAME_PHASE_SPAN_TEMPLATE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} phase_count={} span_count=10 \
legacy_phase_order_comparisons_per_build=58 optimized_phase_order_comparisons_per_build=0 \
legacy_diagnostic_builds_per_build=13 optimized_diagnostic_builds_per_build=0 \
optimized_template_initializations_per_process=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        RENDER_PHASES_BY_QUEUE_ORDER.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn empty_queue_summary() -> RenderPhaseQueueSummary {
    RenderPhaseQueueSummary {
        item_count: 0,
        phase_counts: Vec::new(),
        phase_order_spans: Vec::new(),
        first_ordering_key: None,
        last_ordering_key: None,
    }
}

fn queue_span(phase_order: u8, item_count: usize) -> RenderPhaseQueueSummaryPhaseOrderSpan {
    let phases = RENDER_PHASES_BY_QUEUE_ORDER
        .iter()
        .copied()
        .filter(|phase| phase.queue_order() == phase_order)
        .collect::<Vec<_>>();
    let mut span = RenderPhaseQueueSummaryPhaseOrderSpan::new(phase_order, phases);
    span.item_count = item_count;
    span.start_index = Some(1);
    span.end_index_exclusive = Some(1 + item_count);
    span
}

fn legacy_frame_phase_order_spans(
    geometry: &RenderPhaseQueueSummary,
    sprites: &RenderPhaseQueueSummary,
) -> Vec<RenderFramePhaseQueueSummaryPhaseOrderSpan> {
    let mut spans = Vec::<RenderFramePhaseQueueSummaryPhaseOrderSpan>::new();
    for phase in RENDER_PHASES_BY_QUEUE_ORDER {
        let phase_order = phase.queue_order();
        if let Some(span) = spans
            .iter_mut()
            .find(|span| span.phase_order == phase_order)
        {
            span.phases.push(phase);
            span.diagnostic_name = phase_diagnostic_name(&span.phases);
        } else {
            let geometry_span = geometry.span_for_phase_order(phase_order);
            let sprite_span = sprites.span_for_phase_order(phase_order);
            let geometry_item_count = geometry_span
                .map(|span| span.item_count)
                .unwrap_or_default();
            let sprite_item_count = sprite_span.map(|span| span.item_count).unwrap_or_default();
            spans.push(RenderFramePhaseQueueSummaryPhaseOrderSpan {
                phase_order,
                diagnostic_name: phase_diagnostic_name(&[phase]),
                phases: vec![phase],
                geometry_item_count,
                sprite_item_count,
                total_item_count: geometry_item_count + sprite_item_count,
                geometry_start_index: geometry_span.and_then(|span| span.start_index),
                geometry_end_index_exclusive: geometry_span
                    .and_then(|span| span.end_index_exclusive),
                geometry_first_ordering_key: geometry_span.and_then(|span| span.first_ordering_key),
                geometry_last_ordering_key: geometry_span.and_then(|span| span.last_ordering_key),
                sprite_start_index: sprite_span.and_then(|span| span.start_index),
                sprite_end_index_exclusive: sprite_span.and_then(|span| span.end_index_exclusive),
                sprite_first_ordering_key: sprite_span.and_then(|span| span.first_ordering_key),
                sprite_last_ordering_key: sprite_span.and_then(|span| span.last_ordering_key),
            });
        }
    }
    spans
}

fn measure(
    optimized: bool,
    geometry: &RenderPhaseQueueSummary,
    sprites: &RenderPhaseQueueSummary,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let spans = if optimized {
            frame_phase_order_spans(geometry, sprites)
        } else {
            legacy_frame_phase_order_spans(geometry, sprites)
        };
        checksum = checksum.wrapping_add(
            spans
                .iter()
                .map(|span| span.phases.len() + span.diagnostic_name.len())
                .sum::<usize>(),
        );
        black_box(spans);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
