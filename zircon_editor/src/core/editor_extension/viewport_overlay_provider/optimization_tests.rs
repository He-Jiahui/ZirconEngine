use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

use super::{
    ViewportOverlayProvider, ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};

const CAPABILITY_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 32;

struct EmptyProvider;

impl ViewportOverlayProvider for EmptyProvider {
    fn extract(
        &self,
        _context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        Vec::new()
    }
}

fn registration() -> ViewportOverlayProviderRegistration {
    ViewportOverlayProviderRegistration::new("fixture.overlay", || {
        Arc::new(EmptyProvider) as Arc<dyn ViewportOverlayProvider>
    })
}

fn fixture_capabilities() -> Vec<String> {
    (0..CAPABILITY_COUNT)
        .map(|index| format!("capability.{:04}", index % (CAPABILITY_COUNT / 2)))
        .collect()
}

fn legacy_registration(capabilities: &[String]) -> ViewportOverlayProviderRegistration {
    let mut registration = registration();
    registration
        .required_capabilities
        .extend(capabilities.iter().cloned());
    registration.required_capabilities.sort();
    registration.required_capabilities.dedup();
    registration
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn editor06_viewport_overlay_capability_normalization_preserves_ordered_set() {
    let capabilities = fixture_capabilities();
    let legacy = legacy_registration(&capabilities);
    let optimized = registration().with_required_capabilities(capabilities);

    assert_eq!(
        optimized.required_capabilities(),
        legacy.required_capabilities()
    );
    assert!(optimized
        .required_capabilities()
        .windows(2)
        .all(|window| window[0] < window[1]));
}

#[test]
fn editor06_viewport_overlay_capability_normalization_source_contract() {
    let source = include_str!("../viewport_overlay_provider.rs");
    assert!(source.contains("self.required_capabilities.reserve(lower_bound)"));
    assert!(source.contains("self.required_capabilities.sort_unstable()"));
    assert!(!source.contains("self.required_capabilities.sort();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_viewport_overlay_capability_normalization_bench() {
    let capabilities = fixture_capabilities();
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_registration(&capabilities));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(registration().with_required_capabilities(capabilities.iter().cloned()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "EDITOR06_VIEWPORT_OVERLAY_CAPABILITY_NORMALIZATION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} capabilities={} unique_capabilities={} reserved_slots=0->{} stable_sort=1->0",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        CAPABILITY_COUNT,
        CAPABILITY_COUNT / 2,
        CAPABILITY_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized p95 should be at most 95% of legacy p95"
    );
}
