use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind, UiNodeDefinition,
};

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hw_editor_diff_clones_share_target_document() {
    let before = benchmark_document(1, 64, "before");
    let after = benchmark_document(64, 256, "after");
    let diff = UiAssetDocumentDiff::between(&before, &after);

    let cloned = diff.clone();

    assert!(Arc::ptr_eq(
        diff.target.as_ref().expect("diff target"),
        cloned.target.as_ref().expect("cloned diff target")
    ));
    let mut patched = before;
    assert!(cloned.apply_to(&mut patched));
    assert_eq!(patched, after);
}

#[test]
fn optimization_batch_20260828hw_editor_document_diff_uses_shared_target() {
    let source = include_str!("../document_diff.rs");

    assert!(source.contains("target: Option<Arc<UiAssetDocument>>"));
    assert!(source.contains("then(|| Arc::new(target.clone()))"));
    assert!(source.contains("*document = target.as_ref().clone();"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hw_editor_shared_undo_document_target_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 128;
    let before = benchmark_document(1, 64, "before");
    let target = benchmark_document(1_024, 4 * 1024, "target");
    let legacy = LegacyDocumentDiff {
        target: Some(target.clone()),
    };
    let optimized = UiAssetDocumentDiff::between(&before, &target);

    black_box(legacy.clone());
    black_box(optimized.clone());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(<LegacyDocumentDiff as Clone>::clone(black_box(&legacy)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(<UiAssetDocumentDiff as Clone>::clone(black_box(&optimized)));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR215_SHARED_UNDO_DOCUMENT_TARGET_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

#[derive(Clone)]
struct LegacyDocumentDiff {
    target: Option<UiAssetDocument>,
}

fn benchmark_document(item_count: usize, item_bytes: usize, label: &str) -> UiAssetDocument {
    let payload = "x".repeat(item_bytes);
    let props = (0..item_count)
        .map(|index| {
            (
                format!("property_{index}"),
                Value::String(format!("{label}-{index}-{payload}")),
            )
        })
        .collect::<BTreeMap<_, _>>();
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: format!("editor.test.shared_diff.{label}"),
            version: 1,
            display_name: label.to_string(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: Some(UiNodeDefinition {
            node_id: "root".to_string(),
            widget_type: Some("VerticalBox".to_string()),
            props,
            ..UiNodeDefinition::default()
        }),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
