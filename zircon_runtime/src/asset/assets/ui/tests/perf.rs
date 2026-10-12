use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::{push_reference, AssetReference, UiIconAsset, UiIconSource, UiIconSourceKind};

const SAMPLE_PAIRS: usize = 17;
const RESOLUTIONS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_r6_wave2_runtime635_preserves_single_icon_reference_semantics() {
    let external = icon(
        UiIconSourceKind::SvgAsset,
        Some("res://icons/tool.svg#symbol"),
    );
    let references = external.direct_references();
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].locator.to_string(), "res://icons/tool.svg");

    assert!(icon(UiIconSourceKind::Svg, None)
        .direct_references()
        .is_empty());
    assert!(icon(UiIconSourceKind::Bitmap, None)
        .direct_references()
        .is_empty());
}

#[test]
fn optimization_batch_r6_wave2_runtime635_single_icon_path_has_no_hash_set() {
    let source = include_str!("../../ui.rs");
    let icon_impl = source
        .split_once("impl UiIconAsset")
        .expect("UiIconAsset implementation")
        .1
        .split_once("impl UiV2ViewAsset")
        .expect("next UI asset implementation")
        .0;
    assert!(icon_impl.contains("asset_locator_from_uri(uri)"));
    assert!(!icon_impl.contains("HashSet"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave2_runtime635_single_icon_reference_without_hash_p95() {
    let asset = icon(
        UiIconSourceKind::Bitmap,
        Some("res://icons/tool.png#preview"),
    );
    assert_eq!(asset.direct_references(), legacy_direct_references(&asset));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&asset, false));
            optimized_samples.push(measure(&asset, true));
        } else {
            optimized_samples.push(measure(&asset, true));
            legacy_samples.push(measure(&asset, false));
        }
    }

    let legacy_p95 = p95(&legacy_samples);
    let optimized_p95 = p95(&optimized_samples);
    println!(
        "RUNTIME635_SINGLE_ICON_REFERENCE_WITHOUT_HASH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "single icon reference projection without a hash set must be at least 15% faster at P95"
    );
}

fn icon(kind: UiIconSourceKind, uri: Option<&str>) -> UiIconAsset {
    UiIconAsset {
        source: UiIconSource {
            kind,
            text: None,
            uri: uri.map(str::to_string),
        },
        default_size: 16.0,
        semantic_id: "test.icon".to_string(),
    }
}

fn measure(asset: &UiIconAsset, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..RESOLUTIONS_PER_SAMPLE {
        let references = if optimized {
            asset.direct_references()
        } else {
            legacy_direct_references(asset)
        };
        checksum ^= black_box(references.len() ^ references.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_direct_references(asset: &UiIconAsset) -> Vec<AssetReference> {
    let mut references = Vec::new();
    let mut seen = HashSet::new();
    match asset.source.kind {
        UiIconSourceKind::Svg => {}
        UiIconSourceKind::SvgAsset | UiIconSourceKind::Bitmap => {
            if let Some(uri) = asset.source.uri.as_deref() {
                push_reference(uri, &mut references, &mut seen);
            }
        }
    }
    references
}

fn p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}
