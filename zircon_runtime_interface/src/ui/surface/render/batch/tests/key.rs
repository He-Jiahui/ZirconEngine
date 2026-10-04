use std::{hint::black_box, time::Instant};

use super::*;
use crate::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiGeometry},
    surface::{UiClipMode, UiPaintEffects},
};

const SAMPLE_COUNT: usize = 11;

fn clipped_element() -> UiPaintElement {
    UiPaintElement {
        node_id: UiNodeId::new(7),
        geometry: UiGeometry::from_frame(UiFrame::new(0.0, 0.0, 20.0, 20.0)),
        clip: Some(UiClipState {
            mode: UiClipMode::Scissor,
            frame: UiFrame::new(2.0, 3.0, 10.0, 11.0),
        }),
        z_index: 0,
        paint_order: 0,
        payload: UiPaintPayload::Empty,
        effects: UiPaintEffects::default(),
        cache_generation: None,
        debug_label: None,
    }
}

#[test]
fn runtime_interface03_batch7_standalone_key_matches_shared_clip_interning() {
    let element = clipped_element();
    let direct = UiBatchKey::from_paint_element(&element);
    let mut clip_states = UiBatchClipStates::default();
    let interned = UiBatchKey::from_paint_element_with_clip_states(&element, &mut clip_states);

    assert_eq!(direct, interned);
}

#[test]
#[ignore = "release-only standalone batch-key clip benchmark"]
fn runtime_interface03_batch7_standalone_batch_key_clip_release_benchmark() {
    const KEY_COUNT: usize = 200_000;
    let element = clipped_element();
    let mut interning_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut direct_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_interning = || {
            let started = Instant::now();
            for _ in 0..KEY_COUNT {
                let mut clip_states = UiBatchClipStates::default();
                black_box(UiBatchKey::from_paint_element_with_clip_states(
                    black_box(&element),
                    &mut clip_states,
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_direct = || {
            let started = Instant::now();
            for _ in 0..KEY_COUNT {
                black_box(UiBatchKey::from_paint_element(black_box(&element)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            interning_samples.push(measure_interning());
            direct_samples.push(measure_direct());
        } else {
            direct_samples.push(measure_direct());
            interning_samples.push(measure_interning());
        }
    }

    interning_samples.sort_unstable();
    direct_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STANDALONE_BATCH_KEY_CLIP_BENCH_V1 keys={KEY_COUNT} samples={SAMPLE_COUNT} interning_p50_ns={} direct_p50_ns={} interning_p95_ns={} direct_p95_ns={}",
        interning_samples[p50],
        direct_samples[p50],
        interning_samples[p95],
        direct_samples[p95],
    );
    assert!(
        direct_samples[p95].saturating_mul(5) <= interning_samples[p95].saturating_mul(4),
        "direct standalone clip projection must improve P95 by at least 20%: interning={}ns direct={}ns",
        interning_samples[p95],
        direct_samples[p95],
    );
}
