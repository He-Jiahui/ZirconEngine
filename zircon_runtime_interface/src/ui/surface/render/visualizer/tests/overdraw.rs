use std::{hint::black_box, time::Instant};

use super::*;

const SAMPLE_COUNT: usize = 11;

fn linear_overdraw_regions(
    visible_elements: &[(UiNodeId, UiFrame)],
) -> Vec<UiRenderVisualizerOverdrawRegion> {
    let mut regions: Vec<UiRenderVisualizerOverdrawRegion> = Vec::new();
    for left_index in 0..visible_elements.len() {
        for right_index in left_index + 1..visible_elements.len() {
            let (left_node_id, left_frame) = visible_elements[left_index];
            let (_, right_frame) = visible_elements[right_index];
            if let Some(frame) = left_frame.intersection(right_frame) {
                let mut node_ids = vec![left_node_id];
                for (node_id, candidate_frame) in visible_elements {
                    if candidate_frame.intersection(frame).is_some() {
                        node_ids.push(*node_id);
                    }
                }
                node_ids.sort();
                node_ids.dedup();
                let paint_count = node_ids.len();
                if regions.iter().any(|region| region.frame == frame) {
                    continue;
                }
                regions.push(UiRenderVisualizerOverdrawRegion {
                    frame,
                    paint_count,
                    node_ids,
                    heat: paint_count as f32,
                });
            }
        }
    }
    regions
}

#[test]
fn runtime_interface03_batch6_indexed_overdraw_preserves_linear_output() {
    let visible_elements = vec![
        (UiNodeId::new(4), UiFrame::new(0.0, 0.0, 20.0, 20.0)),
        (UiNodeId::new(2), UiFrame::new(5.0, 5.0, 20.0, 20.0)),
        (UiNodeId::new(4), UiFrame::new(8.0, 8.0, 10.0, 10.0)),
        (UiNodeId::new(9), UiFrame::new(50.0, 50.0, 4.0, 4.0)),
    ];

    assert_eq!(
        overdraw_regions_from_visible(&visible_elements),
        linear_overdraw_regions(&visible_elements),
    );
}

#[test]
fn runtime_interface03_batch6_frame_index_preserves_signed_zero_equality() {
    let mut index = OverdrawFrameIndex::default();
    let negative_zero = UiFrame::new(-0.0, 0.0, 2.0, 2.0);
    let positive_zero = UiFrame::new(0.0, -0.0, 2.0, 2.0);
    let mut regions = Vec::new();

    assert!(index.insert_if_absent(negative_zero, &regions));
    regions.push(UiRenderVisualizerOverdrawRegion {
        frame: negative_zero,
        paint_count: 0,
        node_ids: Vec::new(),
        heat: 0.0,
    });
    assert!(!index.insert_if_absent(positive_zero, &regions));
}

#[test]
#[ignore = "release-only visualizer overdraw frame index benchmark"]
fn runtime_interface03_batch6_visualizer_overdraw_frame_index_release_benchmark() {
    const ELEMENT_COUNT: usize = 192;
    let visible_elements = (0..ELEMENT_COUNT)
        .map(|index| {
            (
                UiNodeId::new(index as u64),
                UiFrame::new(0.0, 0.0, 100.0, 100.0),
            )
        })
        .collect::<Vec<_>>();
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            black_box(linear_overdraw_regions(black_box(&visible_elements)));
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            black_box(overdraw_regions_from_visible(black_box(&visible_elements)));
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
        "RUNTIME_INTERFACE03_VISUALIZER_OVERDRAW_FRAME_INDEX_BENCH_V1 elements={ELEMENT_COUNT} samples={SAMPLE_COUNT} linear_p50_ns={} indexed_p50_ns={} linear_p95_ns={} indexed_p95_ns={}",
        linear_samples[p50],
        indexed_samples[p50],
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(5) <= linear_samples[p95].saturating_mul(4),
        "indexed overdraw admission must improve P95 by at least 20%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}
