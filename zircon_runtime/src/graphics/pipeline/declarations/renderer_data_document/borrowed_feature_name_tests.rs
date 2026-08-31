use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const VALIDATIONS_PER_SAMPLE: usize = 64;
const FEATURES_PER_VALIDATION: usize = 4_096;

#[test]
fn optimization_batch_20260826gn_runtime234_borrowed_names_preserve_duplicate_error() {
    let features = ["lighting", "shadows", "lighting"].map(feature);

    let error = validate_unique_feature_names(&features).expect_err("duplicate must fail");

    assert!(matches!(
        error,
        RendererDataDocumentError::DuplicateRenderFeature { feature }
            if feature == "lighting"
    ));
}

#[test]
fn optimization_batch_20260826gn_runtime234_feature_validation_borrows_names() {
    let source = include_str!("../renderer_data_document.rs");

    assert!(source.contains("seen_features.insert(feature.name.as_str())"));
    assert!(!source.contains("seen_features.insert(feature.name.clone())"));
    assert!(source.contains("feature: feature.name.clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gn_runtime234_renderer_feature_name_borrowed_set_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME234_RENDERER_FEATURE_NAME_BORROWED_SET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
validations_per_sample={VALIDATIONS_PER_SAMPLE} features_per_validation={FEATURES_PER_VALIDATION} \
legacy_owned_name_clones_per_validation={FEATURES_PER_VALIDATION} \
optimized_owned_name_clones_per_validation=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn feature(name: &str) -> RendererFeatureDocument {
    RendererFeatureDocument {
        name: name.to_string(),
        source: "builtin".to_string(),
        enabled: true,
        quality_gate: None,
        shader: None,
        material: None,
        required_entry_points: Vec::new(),
        expected_properties: Vec::new(),
        expected_texture_slots: Vec::new(),
        local_config: BTreeMap::new(),
    }
}

fn measure(borrow_names: bool) -> u128 {
    let names = (0..FEATURES_PER_VALIDATION)
        .map(|index| format!("renderer-feature-{index:04}"))
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut checksum = 0usize;
    for validation in 0..VALIDATIONS_PER_SAMPLE {
        if borrow_names {
            let mut seen = BTreeSet::new();
            for name in &names {
                seen.insert(black_box(name.as_str()));
            }
            checksum ^= black_box(seen.len() ^ validation);
            black_box(seen);
        } else {
            let mut seen = BTreeSet::new();
            for name in &names {
                seen.insert(black_box(name.clone()));
            }
            checksum ^= black_box(seen.len() ^ validation);
            black_box(seen);
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
