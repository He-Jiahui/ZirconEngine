use crate::ui::surface::render::{
    batch_indices_by_source_index, UiBatch, UiBatchKey, UiBatchRange, UiBatchSplitReason,
    UiBatchPrimitive, UiBatchShader, UiOpacityClass,
};

fn batch(source_indices: Vec<usize>) -> UiBatch {
    UiBatch {
        layer: 0,
        key: UiBatchKey {
            clip: None,
            primitive: UiBatchPrimitive::Empty,
            shader: UiBatchShader::None,
            resource: None,
            text_backend: None,
            draw_effects: Vec::new(),
            opacity_class: UiOpacityClass::Opaque,
        },
        range: UiBatchRange::default(),
        source_indices,
        node_ids: Vec::new(),
        split_reason: UiBatchSplitReason::FirstBatch,
    }
}

#[test]
fn first_batch_wins_for_duplicate_source_indices() {
    let batches = vec![batch(vec![2, 4]), batch(vec![1, 2])];

    assert_eq!(
        batch_indices_by_source_index(&batches, 5),
        vec![None, Some(1), Some(0), None, Some(0)]
    );
}

#[test]
fn ignores_out_of_range_source_indices() {
    let batches = vec![batch(vec![0, 9])];

    assert_eq!(
        batch_indices_by_source_index(&batches, 2),
        vec![Some(0), None]
    );
}

fn separate_stats(rows: &[(bool, bool, bool)]) -> (usize, usize, usize) {
    (
        rows.iter().filter(|row| row.0).count(),
        rows.iter().filter(|row| row.1).count(),
        rows.iter().filter(|row| row.2).count(),
    )
}

fn fused_stats(rows: &[(bool, bool, bool)]) -> (usize, usize, usize) {
    let mut clipped = 0;
    let mut resource_bound = 0;
    let mut text = 0;
    for &(has_clip, has_resource, is_text) in rows {
        clipped += usize::from(has_clip);
        resource_bound += usize::from(has_resource);
        text += usize::from(is_text);
    }
    (clipped, resource_bound, text)
}

#[test]
fn fused_stats_match_separate_scans() {
    let rows = [(true, false, false), (false, true, false), (true, true, true)];

    assert_eq!(fused_stats(&rows), separate_stats(&rows));
}

#[test]
#[ignore = "release-only renderer parity stats fusion benchmark"]
fn renderer_parity_stats_fusion_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ROW_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 11;
    let rows: Vec<_> = (0..ROW_COUNT)
        .map(|index| (index % 2 == 0, index % 3 == 0, index % 5 == 0))
        .collect();
    let mut separate_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut fused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_separate = || {
            let started = Instant::now();
            black_box(separate_stats(&rows));
            started.elapsed().as_nanos()
        };
        let measure_fused = || {
            let started = Instant::now();
            black_box(fused_stats(&rows));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            separate_samples.push(measure_separate());
            fused_samples.push(measure_fused());
        } else {
            fused_samples.push(measure_fused());
            separate_samples.push(measure_separate());
        }
    }

    separate_samples.sort_unstable();
    fused_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_RENDER_PARITY_STATS_FUSION_BENCH_V1 rows={ROW_COUNT} samples={SAMPLE_COUNT} separate_p50_ns={} fused_p50_ns={} separate_p95_ns={} fused_p95_ns={}",
        separate_samples[p50],
        fused_samples[p50],
        separate_samples[p95],
        fused_samples[p95],
    );
    assert!(
        fused_samples[p95].saturating_mul(5) <= separate_samples[p95].saturating_mul(4),
        "fused stats must improve P95 by at least 20%: separate={}ns fused={}ns",
        separate_samples[p95],
        fused_samples[p95],
    );
}

#[test]
#[ignore = "release-only renderer parity batch mapping benchmark"]
fn renderer_parity_batch_mapping_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ELEMENT_COUNT: usize = 4_096;
    const SAMPLE_COUNT: usize = 11;
    let source_indices: Vec<_> = (0..ELEMENT_COUNT).collect();
    let batches: Vec<_> = (0..ELEMENT_COUNT)
        .step_by(8)
        .map(|start| batch((start..(start + 8).min(ELEMENT_COUNT)).collect()))
        .collect();
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for &paint_index in &source_indices {
                black_box(
                    batches
                        .iter()
                        .position(|batch| batch.source_indices.contains(&paint_index)),
                );
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            black_box(batch_indices_by_source_index(&batches, ELEMENT_COUNT));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_RENDER_PARITY_BATCH_MAPPING_BENCH_V1 elements={ELEMENT_COUNT} samples={SAMPLE_COUNT} linear_p50_ns={} indexed_p50_ns={} linear_p95_ns={} indexed_p95_ns={}",
        linear_samples[p50],
        indexed_samples[p50],
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(5) <= linear_samples[p95].saturating_mul(4),
        "indexed batch mapping must improve P95 by at least 20%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}
