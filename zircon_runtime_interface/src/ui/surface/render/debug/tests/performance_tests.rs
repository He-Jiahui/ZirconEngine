use std::{hint::black_box, time::Instant};

use crate::ui::{
    event_ui::UiNodeId,
    surface::{
        UiBatch, UiBatchKey, UiBatchPrimitive, UiBatchRange, UiBatchShader, UiBatchSplitReason,
        UiDrawEffect, UiOpacityClass, UiRenderResourceKey, UiRenderResourceKind,
    },
};

use super::UiRenderBatchDebugEntry;

const SAMPLE_COUNT: usize = 11;

fn batch(index: usize) -> UiBatch {
    UiBatch {
        layer: index as i32,
        key: UiBatchKey {
            clip: None,
            primitive: UiBatchPrimitive::Image,
            shader: UiBatchShader::Image,
            resource: Some(UiRenderResourceKey::new(
                UiRenderResourceKind::Image,
                format!("debug-image-{index}"),
            )),
            text_backend: None,
            draw_effects: vec![UiDrawEffect::PixelSnapped, UiDrawEffect::NoGamma],
            opacity_class: UiOpacityClass::Opaque,
        },
        range: UiBatchRange {
            first_element: index * 8,
            element_count: 8,
        },
        source_indices: (index * 8..index * 8 + 8).collect(),
        node_ids: (index * 8..index * 8 + 8)
            .map(|node| UiNodeId::new(node as u64))
            .collect(),
        split_reason: UiBatchSplitReason::ResourceChanged,
    }
}

fn cloned_debug_entries(batches: &[UiBatch]) -> Vec<UiRenderBatchDebugEntry> {
    batches
        .iter()
        .map(|batch| UiRenderBatchDebugEntry {
            layer: batch.layer,
            key: batch.key.clone(),
            first_element: batch.range.first_element,
            element_count: batch.range.element_count,
            source_indices: batch.source_indices.clone(),
            node_ids: batch.node_ids.clone(),
            split_reason: batch.split_reason,
        })
        .collect()
}

fn moved_debug_entries(batches: Vec<UiBatch>) -> Vec<UiRenderBatchDebugEntry> {
    batches
        .into_iter()
        .map(|batch| UiRenderBatchDebugEntry {
            layer: batch.layer,
            key: batch.key,
            first_element: batch.range.first_element,
            element_count: batch.range.element_count,
            source_indices: batch.source_indices,
            node_ids: batch.node_ids,
            split_reason: batch.split_reason,
        })
        .collect()
}

#[test]
fn runtime_interface03_batch8_moved_debug_batches_preserve_entries() {
    let batches = (0..8).map(batch).collect::<Vec<_>>();

    assert_eq!(
        moved_debug_entries(batches.clone()),
        cloned_debug_entries(&batches)
    );
}

#[test]
#[ignore = "release-only moved render-debug batch benchmark"]
fn runtime_interface03_batch8_moved_render_debug_batches_release_benchmark() {
    const BATCH_COUNT: usize = 4_096;
    let source = (0..BATCH_COUNT).map(batch).collect::<Vec<_>>();
    let mut cloned_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut moved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let cloned_input = source.clone();
        let moved_input = source.clone();
        let measure_cloned = || {
            let started = Instant::now();
            black_box(cloned_debug_entries(&cloned_input));
            black_box(cloned_input);
            started.elapsed().as_nanos()
        };
        let measure_moved = || {
            let started = Instant::now();
            black_box(moved_debug_entries(moved_input));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloned_samples.push(measure_cloned());
            moved_samples.push(measure_moved());
        } else {
            moved_samples.push(measure_moved());
            cloned_samples.push(measure_cloned());
        }
    }

    cloned_samples.sort_unstable();
    moved_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_MOVED_RENDER_DEBUG_BATCHES_BENCH_V1 batches={BATCH_COUNT} samples={SAMPLE_COUNT} cloned_p50_ns={} moved_p50_ns={} cloned_p95_ns={} moved_p95_ns={}",
        cloned_samples[p50],
        moved_samples[p50],
        cloned_samples[p95],
        moved_samples[p95],
    );
    assert!(
        moved_samples[p95].saturating_mul(5) <= cloned_samples[p95].saturating_mul(4),
        "moved debug batches must improve P95 by at least 20%: cloned={}ns moved={}ns",
        cloned_samples[p95],
        moved_samples[p95],
    );
}
