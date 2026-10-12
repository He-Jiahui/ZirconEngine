use std::{hint::black_box, time::Instant};

use crate::ui::{layout::UiFrame, surface::render::UiClipMode};

use super::{UiBatchClipStates, UiClipStack, UiClipState};

const CLIP_COUNT: usize = 4_096;
const LOOKUP_COUNT: usize = 256;
const SAMPLE_COUNT: usize = 11;

fn clip(index: usize) -> UiClipState {
    UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(index as f32, index as f32, 8.0, 8.0),
    }
}

fn populated_stack() -> UiClipStack {
    let mut stack = UiClipStack::default();
    for index in 0..CLIP_COUNT {
        stack.states.intern(clip(index));
    }
    stack
}

#[test]
fn runtime_interface03_batch17_clip_resolution_preserves_deserialized_fallback() {
    let stack = populated_stack();
    let target = clip(CLIP_COUNT - 1);
    assert_eq!(stack.resolve(&target), stack.resolve_linear(&target));

    let encoded = serde_json::to_string(&stack.states).unwrap();
    let states: UiBatchClipStates = serde_json::from_str(&encoded).unwrap();
    assert!(states.indices.is_empty());
    let deserialized = UiClipStack {
        states,
        stack: Vec::new(),
    };
    assert_eq!(
        deserialized.resolve(&target),
        deserialized.resolve_linear(&target),
    );
}

#[test]
#[ignore = "release-only indexed clip-state resolution benchmark"]
fn runtime_interface03_batch17_clip_resolution_release_benchmark() {
    let stack = populated_stack();
    let target = clip(CLIP_COUNT - 1);
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(stack.resolve_linear(black_box(&target)));
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(stack.resolve(black_box(&target)));
            }
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
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_CLIP_STATE_RESOLUTION_BENCH_V1 clips={CLIP_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} indexed_p95_ns={}",
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(5) <= linear_samples[p95],
        "indexed clip-state resolution must improve P95 by at least 80%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}
