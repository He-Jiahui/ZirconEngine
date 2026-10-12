use std::{hint::black_box, time::Instant};

use super::{
    UiModelContextLayer, UiModelContextOverride, UiModelContextPatch, UiResolvedModelContext,
};
use crate::ui::binding::model::{UiModelProviderId, UiModelProviderKey, UiModelProviderVersion};

fn provider(label: &str) -> UiModelProviderKey {
    UiModelProviderKey {
        id: UiModelProviderId::try_new(format!("{label}.{}", "x".repeat(192))).unwrap(),
        version: UiModelProviderVersion::try_new(1).unwrap(),
    }
}

fn full_context(prefix: &str) -> UiResolvedModelContext {
    UiResolvedModelContext {
        surface: Some(provider(&format!("{prefix}.surface"))),
        component: Some(provider(&format!("{prefix}.component"))),
        row: Some(provider(&format!("{prefix}.row"))),
        item: Some(provider(&format!("{prefix}.item"))),
    }
}

fn resolve_cloning_parent(
    parent: Option<&UiResolvedModelContext>,
    patch: &UiModelContextPatch,
) -> UiResolvedModelContext {
    let mut resolved = parent.cloned().unwrap_or_default();
    for layer in UiModelContextLayer::ALL {
        match patch.override_for(layer) {
            None => {}
            Some(UiModelContextOverride::Bind { provider }) => {
                *resolved.provider_mut(layer) = Some(provider.clone());
            }
            Some(UiModelContextOverride::Clear) => {
                *resolved.provider_mut(layer) = None;
            }
        }
    }
    resolved
}

#[test]
fn selective_model_context_clone_preserves_every_override_shape() {
    let parent = full_context("parent");
    let patches = [
        UiModelContextPatch::default(),
        UiModelContextPatch::default()
            .with_binding(UiModelContextLayer::Surface, provider("next.surface")),
        UiModelContextPatch::default().with_clear(UiModelContextLayer::Component),
        UiModelContextPatch::default()
            .with_clear(UiModelContextLayer::Surface)
            .with_binding(UiModelContextLayer::Component, provider("next.component"))
            .with_clear(UiModelContextLayer::Row)
            .with_binding(UiModelContextLayer::Item, provider("next.item")),
    ];

    for patch in patches {
        assert_eq!(
            UiResolvedModelContext::resolve(Some(&parent), &patch),
            resolve_cloning_parent(Some(&parent), &patch),
        );
    }
    assert_eq!(
        UiResolvedModelContext::resolve(None, &UiModelContextPatch::default()),
        UiResolvedModelContext::default(),
    );
}

#[test]
#[ignore = "release-only selective model context cloning benchmark"]
fn runtime_interface03_batch40_selective_model_context_clone_release_benchmark() {
    const ITERATIONS: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let parent = full_context("parent");
    let patch = UiModelContextPatch::default()
        .with_binding(UiModelContextLayer::Surface, provider("next.surface"))
        .with_binding(UiModelContextLayer::Component, provider("next.component"))
        .with_binding(UiModelContextLayer::Row, provider("next.row"))
        .with_binding(UiModelContextLayer::Item, provider("next.item"));
    let mut cloning_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut selective_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_cloning = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(resolve_cloning_parent(
                    black_box(Some(&parent)),
                    black_box(&patch),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_selective = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(UiResolvedModelContext::resolve(
                    black_box(Some(&parent)),
                    black_box(&patch),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloning_samples.push(measure_cloning());
            selective_samples.push(measure_selective());
        } else {
            selective_samples.push(measure_selective());
            cloning_samples.push(measure_cloning());
        }
    }

    cloning_samples.sort_unstable();
    selective_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SELECTIVE_MODEL_CONTEXT_CLONE_BENCH_V1 iterations={ITERATIONS} samples={SAMPLE_COUNT} cloning_p95_ns={} selective_p95_ns={}",
        cloning_samples[p95], selective_samples[p95],
    );
    assert!(
        selective_samples[p95].saturating_mul(2) <= cloning_samples[p95],
        "selective model context cloning must improve P95 by at least 50%: cloning={}ns selective={}ns",
        cloning_samples[p95],
        selective_samples[p95],
    );
}
