use std::{hint::black_box, time::Instant};

use crate::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiGeometry},
};

use super::{
    batch_bounds, visualizer_overlays, UiBatchPlan, UiPaintElement, UiPaintPayload,
    UiRenderVisualizerOverlay, UiRenderVisualizerOverlayKind,
};
use crate::ui::surface::UiPaintEffects;

const SAMPLE_COUNT: usize = 11;

fn paint_element(index: usize) -> UiPaintElement {
    UiPaintElement {
        node_id: UiNodeId::new(index as u64),
        geometry: UiGeometry::from_frame(UiFrame::new(index as f32, 0.0, 2.0, 2.0)),
        clip: None,
        z_index: 0,
        paint_order: index as u64,
        payload: UiPaintPayload::Empty,
        effects: UiPaintEffects::default(),
        cache_generation: None,
        debug_label: None,
    }
}

#[inline(never)]
fn projected_batch_bounds(
    elements: &[UiPaintElement],
    source_indices: &[usize],
) -> Option<UiFrame> {
    batch_bounds(elements, source_indices)
}

fn unreserved_wireframe_overlays(elements: &[UiPaintElement]) -> Vec<UiRenderVisualizerOverlay> {
    let mut overlays = Vec::new();
    for (paint_index, element) in elements.iter().enumerate() {
        overlays.push(UiRenderVisualizerOverlay {
            kind: UiRenderVisualizerOverlayKind::Wireframe,
            frame: element.geometry.render_bounds,
            node_id: Some(element.node_id),
            paint_index: Some(paint_index),
            batch_index: None,
            label: element.debug_label.clone(),
            color: Some("#40c4ff".to_string()),
            intensity: 1.0,
        });
    }
    overlays
}

fn reserved_wireframe_overlays(elements: &[UiPaintElement]) -> Vec<UiRenderVisualizerOverlay> {
    visualizer_overlays(
        elements,
        &UiBatchPlan::default(),
        &vec![None; elements.len()],
        &[],
    )
}

#[test]
fn runtime_interface03_batch24_reserved_visualizer_overlays_preserve_plain_output() {
    let elements = (0..257).map(paint_element).collect::<Vec<_>>();
    assert_eq!(
        reserved_wireframe_overlays(&elements),
        unreserved_wireframe_overlays(&elements),
    );
}

#[test]
#[ignore = "release-only single visualizer batch bounds benchmark"]
fn runtime_interface03_batch6_visualizer_single_batch_bounds_release_benchmark() {
    const ELEMENT_COUNT: usize = 4_096;
    const BATCH_SIZE: usize = 8;
    let elements = (0..ELEMENT_COUNT).map(paint_element).collect::<Vec<_>>();
    let batches = (0..ELEMENT_COUNT)
        .step_by(BATCH_SIZE)
        .map(|start| (start..start + BATCH_SIZE).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut duplicated_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut shared_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_duplicated = || {
            let started = Instant::now();
            for source_indices in &batches {
                black_box(projected_batch_bounds(
                    black_box(&elements),
                    black_box(source_indices),
                ));
                black_box(projected_batch_bounds(
                    black_box(&elements),
                    black_box(source_indices),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_shared = || {
            let started = Instant::now();
            for source_indices in &batches {
                let frame = projected_batch_bounds(black_box(&elements), black_box(source_indices));
                black_box(frame);
                black_box(frame);
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            duplicated_samples.push(measure_duplicated());
            shared_samples.push(measure_shared());
        } else {
            shared_samples.push(measure_shared());
            duplicated_samples.push(measure_duplicated());
        }
    }

    duplicated_samples.sort_unstable();
    shared_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_VISUALIZER_SINGLE_BATCH_BOUNDS_BENCH_V1 elements={ELEMENT_COUNT} batches={} samples={SAMPLE_COUNT} duplicated_p50_ns={} shared_p50_ns={} duplicated_p95_ns={} shared_p95_ns={}",
        batches.len(),
        duplicated_samples[p50],
        shared_samples[p50],
        duplicated_samples[p95],
        shared_samples[p95],
    );
    assert!(
        shared_samples[p95].saturating_mul(5) <= duplicated_samples[p95].saturating_mul(4),
        "single batch-bounds projection must improve P95 by at least 20%: duplicated={}ns shared={}ns",
        duplicated_samples[p95],
        shared_samples[p95],
    );
}

#[test]
#[ignore = "release-only visualizer overlay capacity benchmark"]
fn runtime_interface03_batch24_visualizer_overlay_capacity_release_benchmark() {
    const ELEMENT_COUNT: usize = 4_097;
    let elements = (0..ELEMENT_COUNT).map(paint_element).collect::<Vec<_>>();
    let reserved = reserved_wireframe_overlays(&elements);
    let unreserved = unreserved_wireframe_overlays(&elements);
    assert_eq!(reserved, unreserved);
    assert_eq!(reserved.len(), ELEMENT_COUNT);
    assert_eq!(reserved.capacity(), ELEMENT_COUNT);
    assert!(
        reserved.capacity().saturating_mul(3) <= unreserved.capacity().saturating_mul(2),
        "reserved overlay capacity must be at least 33% below unreserved growth: reserved={} unreserved={}",
        reserved.capacity(),
        unreserved.capacity(),
    );

    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_reserved = || {
            let started = Instant::now();
            black_box(reserved_wireframe_overlays(black_box(&elements)));
            started.elapsed().as_nanos()
        };
        let measure_unreserved = || {
            let started = Instant::now();
            black_box(unreserved_wireframe_overlays(black_box(&elements)));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            reserved_samples.push(measure_reserved());
            unreserved_samples.push(measure_unreserved());
        } else {
            unreserved_samples.push(measure_unreserved());
            reserved_samples.push(measure_reserved());
        }
    }

    reserved_samples.sort_unstable();
    unreserved_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_VISUALIZER_OVERLAY_CAPACITY_BENCH_V1 elements={ELEMENT_COUNT} samples={SAMPLE_COUNT} reserved_capacity={} unreserved_capacity={} reserved_p95_ns={} unreserved_p95_ns={}",
        reserved.capacity(),
        unreserved.capacity(),
        reserved_samples[p95],
        unreserved_samples[p95],
    );
}
