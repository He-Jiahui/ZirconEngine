use super::super::{UiBatch, UiBatchRange, UiOpacityClass};
use super::*;
use crate::ui::surface::{UiShapedTextLine, UiTextDirection, UiTextRange, UiTextWritingMode};

fn shaped_line() -> UiShapedTextLine {
    UiShapedTextLine {
        text: "A".to_string(),
        frame: UiFrame::new(10.0, 20.0, 18.0, 40.0),
        source_range: UiTextRange { start: 0, end: 1 },
        visual_range: UiTextRange { start: 0, end: 1 },
        measured_width: 32.0,
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        ellipsized: false,
        glyphs: Vec::new(),
        clusters: Vec::new(),
    }
}

#[test]
fn text_baseline_overlay_uses_writing_mode_cross_axis() {
    let line = shaped_line();

    assert_eq!(
        text_baseline_overlay_frame(&line, UiTextWritingMode::HorizontalTb),
        UiFrame::new(10.0, 29.0, 32.0, 1.0)
    );
    assert_eq!(
        text_baseline_overlay_frame(&line, UiTextWritingMode::VerticalRl),
        UiFrame::new(19.0, 20.0, 1.0, 32.0)
    );
}

#[test]
fn indexed_cache_statuses_preserve_first_match_and_ignore_unknown() {
    let paint_entries = vec![
        UiRenderCachePaintEntry {
            node_id: UiNodeId::new(1),
            paint_index: 1,
            cache_generation: None,
            status: UiRenderCacheStatus::Rebuilt,
            reason: UiRenderCacheInvalidationReason::NodeDirty,
        },
        UiRenderCachePaintEntry {
            node_id: UiNodeId::new(2),
            paint_index: 1,
            cache_generation: Some(2),
            status: UiRenderCacheStatus::Reused,
            reason: UiRenderCacheInvalidationReason::Unchanged,
        },
        UiRenderCachePaintEntry {
            node_id: UiNodeId::new(3),
            paint_index: 99,
            cache_generation: None,
            status: UiRenderCacheStatus::Rebuilt,
            reason: UiRenderCacheInvalidationReason::ForcedRebuild,
        },
    ];
    assert_eq!(
        paint_cache_statuses_by_index(&paint_entries, 3),
        vec![None, Some(UiRenderCacheStatus::Rebuilt), None]
    );

    let batch_entries = vec![
        UiRenderCacheBatchEntry {
            batch_index: 0,
            batch_key: UiBatchKey {
                clip: None,
                primitive: UiBatchPrimitive::Empty,
                shader: UiBatchShader::None,
                resource: None,
                text_backend: None,
                draw_effects: Vec::new(),
                opacity_class: super::super::UiOpacityClass::Opaque,
            },
            node_ids: Vec::new(),
            status: UiRenderCacheStatus::Reused,
            reason: UiRenderCacheInvalidationReason::Unchanged,
        },
        UiRenderCacheBatchEntry {
            batch_index: 0,
            batch_key: UiBatchKey {
                clip: None,
                primitive: UiBatchPrimitive::Empty,
                shader: UiBatchShader::None,
                resource: None,
                text_backend: None,
                draw_effects: Vec::new(),
                opacity_class: super::super::UiOpacityClass::Opaque,
            },
            node_ids: Vec::new(),
            status: UiRenderCacheStatus::Rebuilt,
            reason: UiRenderCacheInvalidationReason::ForcedRebuild,
        },
    ];
    assert_eq!(
        batch_cache_statuses_by_index(&batch_entries, 1),
        vec![Some((
            UiRenderCacheStatus::Reused,
            UiRenderCacheInvalidationReason::Unchanged,
        ))]
    );
}

#[test]
#[ignore = "release-only render visualizer batch index benchmark"]
fn render_visualizer_batch_index_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ELEMENT_COUNT: usize = 4_096;
    const BATCH_SIZE: usize = 8;
    const SAMPLE_COUNT: usize = 11;
    let batches = (0..ELEMENT_COUNT)
        .step_by(BATCH_SIZE)
        .map(|start| (start..(start + BATCH_SIZE).min(ELEMENT_COUNT)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let batch_plan = batches
        .iter()
        .map(|source_indices| UiBatch {
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
            source_indices: source_indices.clone(),
            node_ids: Vec::new(),
            split_reason: UiBatchSplitReason::FirstBatch,
        })
        .collect::<Vec<_>>();
    let paint_entries = (0..ELEMENT_COUNT)
        .map(|paint_index| UiRenderCachePaintEntry {
            node_id: UiNodeId::new(paint_index as u64),
            paint_index,
            cache_generation: None,
            status: UiRenderCacheStatus::Rebuilt,
            reason: UiRenderCacheInvalidationReason::NodeDirty,
        })
        .collect::<Vec<_>>();
    let batch_entries = (0..batches.len())
        .map(|batch_index| UiRenderCacheBatchEntry {
            batch_index,
            batch_key: UiBatchKey {
                clip: None,
                primitive: UiBatchPrimitive::Empty,
                shader: UiBatchShader::None,
                resource: None,
                text_backend: None,
                draw_effects: Vec::new(),
                opacity_class: super::super::UiOpacityClass::Opaque,
            },
            node_ids: Vec::new(),
            status: UiRenderCacheStatus::Reused,
            reason: UiRenderCacheInvalidationReason::Unchanged,
        })
        .collect::<Vec<_>>();

    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for paint_index in 0..ELEMENT_COUNT {
                black_box(
                    batches
                        .iter()
                        .position(|batch| batch.contains(&paint_index)),
                );
                black_box(
                    paint_entries
                        .iter()
                        .find(|entry| entry.paint_index == paint_index)
                        .map(|entry| entry.status),
                );
            }
            for batch_index in 0..batches.len() {
                black_box(
                    batch_entries
                        .iter()
                        .find(|entry| entry.batch_index == batch_index)
                        .map(|entry| (entry.status, entry.reason)),
                );
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            black_box(batch_indices_by_source_index(&batch_plan, ELEMENT_COUNT));
            black_box(paint_cache_statuses_by_index(&paint_entries, ELEMENT_COUNT));
            black_box(batch_cache_statuses_by_index(&batch_entries, batches.len()));
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
        "RUNTIME_INTERFACE03_RENDER_VISUALIZER_BATCH_INDEX_BENCH_V1 elements={ELEMENT_COUNT} batches={} samples={SAMPLE_COUNT} linear_p50_ns={} indexed_p50_ns={} linear_p95_ns={} indexed_p95_ns={}",
        batches.len(),
        linear_samples[p50],
        indexed_samples[p50],
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(5) <= linear_samples[p95].saturating_mul(4),
        "indexed visualizer lookup must improve P95 by at least 20%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}
