use std::hint::black_box;
use std::time::Instant;

use super::{
    phase_diagnostic_name, phase_order_spans, RenderPhaseQueueSummaryPhaseOrderSpan,
    RENDER_PHASES_BY_QUEUE_ORDER,
};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 20_000;

#[test]
fn optimization_batch_20260829w_runtime296_ordered_span_build_preserves_groups() {
    let spans = phase_order_spans();
    let flattened = spans
        .iter()
        .flat_map(|span| span.phases.iter().copied())
        .collect::<Vec<_>>();

    assert_eq!(spans.len(), 10);
    assert_eq!(flattened, RENDER_PHASES_BY_QUEUE_ORDER);
    assert!(spans
        .windows(2)
        .all(|pair| pair[0].phase_order < pair[1].phase_order));
    for span in &spans {
        assert!(span
            .phases
            .iter()
            .all(|phase| phase.queue_order() == span.phase_order));
        assert_eq!(span.diagnostic_name, phase_diagnostic_name(&span.phases));
    }

    let mut mutated = phase_order_spans();
    mutated[0].item_count = 7;
    assert_eq!(phase_order_spans()[0].item_count, 0);
}

#[test]
fn optimization_batch_20260829w_runtime296_ordered_span_build_uses_adjacent_groups() {
    let source = include_str!("../phase_queue_summary.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn phase_order_spans")
        .nth(1)
        .and_then(|body| body.split("fn phase_diagnostic_name").next())
        .expect("phase span builder");

    assert!(body.contains("Vec::with_capacity(RENDER_PHASES_BY_QUEUE_ORDER.len())"));
    assert!(body.contains("chunk_by"));
    assert!(!body.contains(".position("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829w_runtime296_ordered_phase_span_build_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME296_ORDERED_PHASE_SPAN_BUILD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
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

fn legacy_phase_order_spans() -> Vec<RenderPhaseQueueSummaryPhaseOrderSpan> {
    let mut spans: Vec<RenderPhaseQueueSummaryPhaseOrderSpan> = Vec::new();
    for phase in RENDER_PHASES_BY_QUEUE_ORDER {
        let phase_order = phase.queue_order();
        if let Some(index) = spans
            .iter()
            .position(|span| span.phase_order == phase_order)
        {
            spans[index].phases.push(phase);
            spans[index].diagnostic_name = phase_diagnostic_name(&spans[index].phases);
        } else {
            spans.push(RenderPhaseQueueSummaryPhaseOrderSpan::new(
                phase_order,
                vec![phase],
            ));
        }
    }
    spans
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let spans = if optimized {
            phase_order_spans()
        } else {
            legacy_phase_order_spans()
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
