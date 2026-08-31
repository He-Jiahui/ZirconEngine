use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_MARKER: &str = "RUNTIME64_UI_RESOURCE_INVALIDATION_HASH_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const SCANS_PER_SAMPLE: usize = 4;
const REFERENCE_COUNT: usize = 512;
const INVALIDATION_COUNT: usize = 512;

fn reference(primary: impl Into<String>, fallback: Option<String>) -> UiResourceRef {
    UiResourceRef {
        kind: UiResourceKind::Image,
        uri: primary.into(),
        fallback: UiResourceFallbackPolicy {
            mode: if fallback.is_some() {
                UiResourceFallbackMode::Placeholder
            } else {
                UiResourceFallbackMode::None
            },
            uri: fallback,
        },
    }
}

fn legacy_contains_any_uri(
    reference: &UiResourceRef,
    uris: &[String],
    scheme_map: &UiResourceResolverSchemeMap,
) -> bool {
    uris.iter().any(|uri| {
        reference.uri == uri.as_str()
            || mapped_runtime_locator_string(&reference.uri, scheme_map).as_deref()
                == Some(uri.as_str())
            || reference.fallback.uri.as_deref().is_some_and(|fallback| {
                fallback == uri.as_str()
                    || mapped_runtime_locator_string(fallback, scheme_map).as_deref()
                        == Some(uri.as_str())
            })
    })
}

fn fixtures() -> (Vec<UiResourceRef>, Vec<String>) {
    let references = (0..REFERENCE_COUNT)
        .map(|index| reference(format!("res://textures/reference_{index}.png"), None))
        .collect();
    let invalidations = (0..INVALIDATION_COUNT)
        .map(|index| format!("res://textures/invalidation_{index}.png"))
        .collect();
    (references, invalidations)
}

fn legacy_scan(
    references: &[UiResourceRef],
    invalidations: &[String],
    scheme_map: &UiResourceResolverSchemeMap,
) -> usize {
    references
        .iter()
        .filter(|reference| legacy_contains_any_uri(reference, invalidations, scheme_map))
        .count()
}

fn optimized_scan(
    references: &[UiResourceRef],
    invalidations: &[String],
    scheme_map: &UiResourceResolverSchemeMap,
) -> usize {
    let invalidation_set = invalidations
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    references
        .iter()
        .filter(|reference| {
            resource_reference_contains_any_uri(reference, &invalidation_set, scheme_map)
        })
        .count()
}

fn sample_ns(mut scan: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut observed = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        observed += black_box(scan());
    }
    black_box(observed);
    started.elapsed().as_nanos()
}

fn percentile(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826bb_resource_invalidation_preserves_order_and_matches() {
    let mut resolver = UiResourceResolver::default();
    let report = resolver.invalidate_uris([
        "  res://textures/a.png  ",
        "",
        "res://textures/b.png",
        "res://textures/a.png",
        "   ",
    ]);
    assert_eq!(
        report.requested_uris,
        vec![
            "res://textures/a.png".to_string(),
            "res://textures/b.png".to_string()
        ]
    );

    let references = [
        reference("res://textures/a.png", None),
        reference(
            "res://textures/primary.png",
            Some("res://textures/b.png".to_string()),
        ),
        reference("res://textures/other.png", None),
    ];
    let invalidations = report.requested_uris;
    let invalidation_set = invalidations
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let scheme_map = UiResourceResolverSchemeMap::default();
    for reference in &references {
        assert_eq!(
            resource_reference_contains_any_uri(reference, &invalidation_set, &scheme_map),
            legacy_contains_any_uri(reference, &invalidations, &scheme_map)
        );
    }
}

#[test]
fn optimization_batch_20260826bb_resource_invalidation_uses_hash_membership() {
    let source = include_str!("../resolver.rs");

    assert!(source.contains("HashMap<String, usize>"));
    assert!(source.contains("HashSet<&str>"));
    assert!(source.contains("requested_uri_set.contains(reference_uri)"));
    assert!(!source.contains("requested_uris.iter().any(|existing| existing == uri)"));
    assert!(!source.contains("uris.iter()\n        .any"));
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260826bb_resource_invalidation_hash_membership_p95() {
    let (references, invalidations) = fixtures();
    let scheme_map = UiResourceResolverSchemeMap::default();
    assert_eq!(
        legacy_scan(&references, &invalidations, &scheme_map),
        optimized_scan(&references, &invalidations, &scheme_map)
    );
    for _ in 0..3 {
        black_box(legacy_scan(&references, &invalidations, &scheme_map));
        black_box(optimized_scan(&references, &invalidations, &scheme_map));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(|| {
                legacy_scan(&references, &invalidations, &scheme_map)
            }));
            optimized_samples.push(sample_ns(|| {
                optimized_scan(&references, &invalidations, &scheme_map)
            }));
        } else {
            optimized_samples.push(sample_ns(|| {
                optimized_scan(&references, &invalidations, &scheme_map)
            }));
            legacy_samples.push(sample_ns(|| {
                legacy_scan(&references, &invalidations, &scheme_map)
            }));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples.clone(), 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction = 100.0 - (optimized_p95 as f64 * 100.0 / legacy_p95 as f64);
    println!(
        "{BENCHMARK_MARKER} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} reduction_percent={reduction:.3} references={REFERENCE_COUNT} invalidations={INVALIDATION_COUNT} scans_per_sample={SCANS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS}"
    );

    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(10),
        "expected hash invalidation P95 to be at least 90% below nested URI scans; legacy={legacy_p95}ns optimized={optimized_p95}ns reduction={reduction:.3}%"
    );
}
