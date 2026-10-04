use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const FEATURES_PER_SAMPLE: usize = 512;
const ANCHORS_PER_SAMPLE: usize = 4;

#[test]
fn anchor_lookup_preserves_first_occurrence_per_name() {
    let features = vec![
        plugin("bloom"),
        plugin("unrelated"),
        plugin("bloom"),
        plugin("reflection_probes"),
        plugin("reflection_probes"),
    ];

    assert_eq!(
        index_after_last_feature_name(&features, &["bloom", "reflection_probes"]),
        Some(4)
    );
}

#[test]
fn anchor_lookup_scans_features_once() {
    let source = include_str!("../plugin_render_features.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("plugin feature insertion implementation");

    assert!(implementation.contains("let mut matched_names = vec![false; names.len()]"));
    assert!(implementation.contains("for (feature_index, feature) in features.iter().enumerate()"));
    assert!(implementation.contains("let feature_name = feature.feature_name()"));
    assert!(implementation.contains("if matched_names[name_index]"));
    assert!(
        !implementation.contains(".filter_map(|name| index_after_feature_name(features, name))")
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cm_runtime389_plugin_anchor_single_scan_p95() {
    let features = (0..FEATURES_PER_SAMPLE)
        .map(|index| format!("feature_{index:04}_owned_descriptor_name"))
        .collect::<Vec<_>>();
    let anchors = [
        features[FEATURES_PER_SAMPLE - 8].as_str(),
        features[FEATURES_PER_SAMPLE - 6].as_str(),
        features[FEATURES_PER_SAMPLE - 4].as_str(),
        features[FEATURES_PER_SAMPLE - 2].as_str(),
    ];
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&features, &anchors, false));
            optimized.push(measure(&features, &anchors, true));
        } else {
            optimized.push(measure(&features, &anchors, true));
            legacy.push(measure(&features, &anchors, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME389_PLUGIN_FEATURE_ANCHOR_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} features_per_sample={FEATURES_PER_SAMPLE} anchors_per_sample={ANCHORS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn plugin(name: &str) -> RendererFeatureAsset {
    RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
        name,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

fn measure(features: &[String], anchors: &[&str], use_single_scan: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let selected = if use_single_scan {
            let mut matched = vec![false; anchors.len()];
            let mut selected = None;
            for (feature_index, feature) in black_box(features).iter().enumerate() {
                let owned_name = feature.clone();
                let Some(anchor_index) = anchors.iter().position(|name| *name == owned_name) else {
                    continue;
                };
                if !matched[anchor_index] {
                    matched[anchor_index] = true;
                    selected = Some(feature_index + 1);
                }
            }
            selected
        } else {
            black_box(anchors)
                .iter()
                .filter_map(|name| {
                    features
                        .iter()
                        .position(|feature| feature.clone() == *name)
                        .map(|index| index + 1)
                })
                .max()
        };
        checksum ^= selected.unwrap_or_default();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
