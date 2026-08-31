use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::{
    template::UiAssetImports,
    v2::{UiV2AssetHeader, UiV2StyleSheet},
};

const TOKEN_COUNT: usize = 512;
const OPERATIONS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hk_editor203_preserves_v2_style_merge_order() {
    let mut base = document(UiV2AssetKind::Component, "base");
    base.tokens.insert(
        "shared".to_string(),
        toml::Value::String("base".to_string()),
    );
    base.stylesheets.push(stylesheet("base"));

    let mut style_a = document(UiV2AssetKind::Style, "style-a");
    style_a
        .tokens
        .insert("shared".to_string(), toml::Value::String("a".to_string()));
    style_a
        .tokens
        .insert("only-a".to_string(), toml::Value::Integer(1));
    style_a.stylesheets.push(stylesheet("a"));
    let mut style_b = document(UiV2AssetKind::Style, "style-b");
    style_b
        .tokens
        .insert("shared".to_string(), toml::Value::String("b".to_string()));
    style_b.stylesheets.push(stylesheet("b"));
    let styles = BTreeMap::from([("a".to_string(), style_a), ("b".to_string(), style_b)]);

    let merged = v2_preview_document_with_imported_styles(&base, &styles);

    assert_eq!(merged.tokens["shared"].as_str(), Some("b"));
    assert_eq!(merged.tokens["only-a"].as_integer(), Some(1));
    assert_eq!(
        merged
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        ["base", "a", "b"]
    );
    assert_eq!(base.tokens["shared"].as_str(), Some("base"));
}

#[test]
fn optimization_batch_20260826hk_editor203_streams_v2_style_clones() {
    let source = include_str!("../v2_authoring.rs");
    assert!(source.contains("extend_cloned_map(&mut document.tokens, &style.tokens)"));
    assert!(source.contains("extend_cloned_values(&mut document.stylesheets, &style.stylesheets)"));
    assert!(source.contains(".iter()"));
    assert!(!source.contains("style.tokens.clone()"));
    assert!(!source.contains("style.stylesheets.clone()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hk_editor203_streaming_v2_style_merge_release_benchmark() {
    let source = (0..TOKEN_COUNT)
        .map(|value| (value, value * 2))
        .collect::<BTreeMap<_, _>>();
    let mut legacy = BTreeMap::new();
    let mut optimized = BTreeMap::new();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                legacy.clear();
                legacy_extend_cloned_map(black_box(&mut legacy), black_box(&source));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                optimized.clear();
                extend_cloned_map(black_box(&mut optimized), black_box(&source));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR203_STREAMING_V2_STYLE_MERGE_BENCH_V1 \
         token_count={TOKEN_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn document(kind: UiV2AssetKind, id: &str) -> UiV2AssetDocument {
    UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind,
            id: id.to_string(),
            version: 2,
            display_name: String::new(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: None,
        nodes: BTreeMap::new(),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

fn stylesheet(id: &str) -> UiV2StyleSheet {
    UiV2StyleSheet {
        id: id.to_string(),
        rules: Vec::new(),
    }
}

fn legacy_extend_cloned_map<K, V>(target: &mut BTreeMap<K, V>, source: &BTreeMap<K, V>)
where
    K: Clone + Ord,
    V: Clone,
{
    target.extend(source.clone());
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
