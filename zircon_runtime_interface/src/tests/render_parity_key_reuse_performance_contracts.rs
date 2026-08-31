use crate::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{UiFrame, UiGeometry},
    surface::render::{
        UiBatchKey, UiBatchPlan, UiBrushPayload, UiBrushSet, UiDrawEffect, UiPaintEffects,
        UiPaintElement, UiPaintPayload, UiRenderCommand, UiRenderCommandKind, UiRenderResourceKey,
        UiRenderResourceKind, UiRendererParitySnapshot, UiResolvedStyle,
    },
};

fn image_element(resource: UiRenderResourceKey, effects: Vec<UiDrawEffect>) -> UiPaintElement {
    UiPaintElement {
        node_id: UiNodeId::new(1),
        geometry: UiGeometry::default(),
        clip: None,
        z_index: 0,
        paint_order: 0,
        payload: UiPaintPayload::Brush {
            brushes: UiBrushSet {
                fill: Some(UiBrushPayload::image(resource)),
                border: None,
            },
        },
        effects: UiPaintEffects {
            opacity: 1.0,
            effects,
        },
        cache_generation: None,
        debug_label: None,
    }
}

fn text_element() -> UiPaintElement {
    UiRenderCommand {
        node_id: UiNodeId::new(2),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::default(),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: Some("parity".to_string()),
        image: None,
        opacity: 1.0,
    }
    .to_paint_element(1)
}

#[test]
fn parity_rows_preserve_resource_and_text_metadata() {
    let resource = UiRenderResourceKey::new(UiRenderResourceKind::Image, "primary")
        .with_fallback(UiRenderResourceKey::new(
            UiRenderResourceKind::Image,
            "fallback",
        ));
    let elements = vec![image_element(resource, Vec::new()), text_element()];
    let plan = UiBatchPlan::from_paint_elements(&elements);
    let snapshot = UiRendererParitySnapshot::from_paint_elements_batches(
        UiTreeId::new("ui.parity"),
        &elements,
        &plan,
    );

    for row in &snapshot.paint_order {
        assert_eq!(row.resource, row.batch_key.resource);
        assert_eq!(row.text_render_mode, row.batch_key.text_backend);
    }
}

#[test]
#[ignore = "release-only renderer parity key reuse benchmark"]
fn renderer_parity_key_reuse_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ELEMENT_COUNT: usize = 4_096;
    const SAMPLE_COUNT: usize = 11;
    let resource = UiRenderResourceKey::new(UiRenderResourceKind::Image, "primary")
        .with_fallback(
            UiRenderResourceKey::new(UiRenderResourceKind::Image, "fallback-1").with_fallback(
                UiRenderResourceKey::new(UiRenderResourceKind::Image, "fallback-2"),
            ),
        );
    let element = image_element(
        resource,
        (0..24)
            .map(|index| match index % 3 {
                0 => UiDrawEffect::PixelSnapped,
                1 => UiDrawEffect::DisabledEffect,
                _ => UiDrawEffect::NoGamma,
            })
            .collect(),
    );
    let elements = vec![element; ELEMENT_COUNT];
    let mut repeated_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_repeated = || {
            let started = Instant::now();
            for element in &elements {
                let batch_key = UiBatchKey::from_paint_element(element);
                let resource = UiBatchKey::from_paint_element(element).resource;
                black_box((batch_key, resource));
            }
            started.elapsed().as_nanos()
        };
        let measure_reused = || {
            let started = Instant::now();
            for element in &elements {
                let batch_key = UiBatchKey::from_paint_element(element);
                let resource = batch_key.resource.clone();
                let text_render_mode = batch_key.text_backend;
                black_box((batch_key, resource, text_render_mode));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            repeated_samples.push(measure_repeated());
            reused_samples.push(measure_reused());
        } else {
            reused_samples.push(measure_reused());
            repeated_samples.push(measure_repeated());
        }
    }

    repeated_samples.sort_unstable();
    reused_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_RENDER_PARITY_KEY_REUSE_BENCH_V1 elements={ELEMENT_COUNT} samples={SAMPLE_COUNT} repeated_p50_ns={} reused_p50_ns={} repeated_p95_ns={} reused_p95_ns={}",
        repeated_samples[p50],
        reused_samples[p50],
        repeated_samples[p95],
        reused_samples[p95],
    );
    assert!(
        reused_samples[p95].saturating_mul(5) <= repeated_samples[p95].saturating_mul(4),
        "key reuse must improve P95 by at least 20%: repeated={}ns reused={}ns",
        repeated_samples[p95],
        reused_samples[p95],
    );
}
