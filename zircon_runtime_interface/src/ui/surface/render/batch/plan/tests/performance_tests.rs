use std::{hint::black_box, time::Instant};

use crate::ui::surface::{
    UiBatchKey, UiBatchPrimitive, UiBatchShader, UiDrawEffect, UiOpacityClass, UiRenderResourceKey,
    UiRenderResourceKind,
};

const SAMPLE_COUNT: usize = 11;

fn batch_key(index: usize) -> UiBatchKey {
    UiBatchKey {
        clip: None,
        primitive: UiBatchPrimitive::Image,
        shader: UiBatchShader::Image,
        resource: Some(
            UiRenderResourceKey::new(UiRenderResourceKind::Image, format!("image-{index}"))
                .with_fallback(UiRenderResourceKey::new(
                    UiRenderResourceKind::Texture,
                    format!("fallback-{index}"),
                )),
        ),
        text_backend: None,
        draw_effects: vec![UiDrawEffect::PixelSnapped, UiDrawEffect::NoGamma],
        opacity_class: UiOpacityClass::Opaque,
    }
}

fn cloned_rollover(keys: &[UiBatchKey]) -> Vec<UiBatchKey> {
    let mut completed = Vec::with_capacity(keys.len());
    let mut active: Option<UiBatchKey> = None;
    for key in keys.iter().cloned() {
        if let Some(current) = active.as_ref() {
            completed.push(current.clone());
        }
        active = Some(key);
    }
    completed.extend(active);
    completed
}

fn moved_rollover(keys: &[UiBatchKey]) -> Vec<UiBatchKey> {
    let mut completed = Vec::with_capacity(keys.len());
    let mut active = None;
    for key in keys.iter().cloned() {
        if let Some(previous) = active.replace(key) {
            completed.push(previous);
        }
    }
    completed.extend(active);
    completed
}

#[test]
fn runtime_interface03_batch7_moved_rollover_preserves_key_order() {
    let keys = (0..8).map(batch_key).collect::<Vec<_>>();

    assert_eq!(moved_rollover(&keys), cloned_rollover(&keys));
}

#[test]
#[ignore = "release-only moved batch-key rollover benchmark"]
fn runtime_interface03_batch7_moved_batch_key_rollover_release_benchmark() {
    const KEY_COUNT: usize = 4_096;
    let keys = (0..KEY_COUNT).map(batch_key).collect::<Vec<_>>();
    let mut cloned_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut moved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_cloned = || {
            let started = Instant::now();
            black_box(cloned_rollover(black_box(&keys)));
            started.elapsed().as_nanos()
        };
        let measure_moved = || {
            let started = Instant::now();
            black_box(moved_rollover(black_box(&keys)));
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
        "RUNTIME_INTERFACE03_MOVED_BATCH_KEY_ROLLOVER_BENCH_V1 keys={KEY_COUNT} samples={SAMPLE_COUNT} cloned_p50_ns={} moved_p50_ns={} cloned_p95_ns={} moved_p95_ns={}",
        cloned_samples[p50],
        moved_samples[p50],
        cloned_samples[p95],
        moved_samples[p95],
    );
    assert!(
        moved_samples[p95].saturating_mul(5) <= cloned_samples[p95].saturating_mul(4),
        "moved batch-key rollover must improve P95 by at least 20%: cloned={}ns moved={}ns",
        cloned_samples[p95],
        moved_samples[p95],
    );
}
