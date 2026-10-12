use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::template::{UiAssetHeader, UiAssetImports};

const IMPORT_COUNT: usize = 512;
const OPERATIONS_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hh_editor200_preserves_preview_import_merge_semantics() {
    let source = BTreeMap::from([
        ("new".to_string(), document("new-source")),
        ("shared".to_string(), document("shared-source")),
    ]);
    let mut target = BTreeMap::from([
        ("kept".to_string(), document("kept-target")),
        ("shared".to_string(), document("shared-target")),
    ]);

    extend_compiler_imports(&mut target, &source);

    assert_eq!(target["kept"].asset.id, "kept-target");
    assert_eq!(target["new"].asset.id, "new-source");
    assert_eq!(target["shared"].asset.id, "shared-source");
    assert_eq!(source["shared"].asset.id, "shared-source");
}

#[test]
fn optimization_batch_20260826hh_editor200_streams_preview_import_clones() {
    let source = include_str!("../preview_compile.rs");
    let start = source
        .find("fn extend_compiler_imports(")
        .expect("extend_compiler_imports function");
    let end = source[start..]
        .find("\npub(super) fn preview_size_for_preset")
        .map(|offset| start + offset)
        .expect("preview_size_for_preset boundary");
    let body = &source[start..end];

    assert!(body.contains(".iter()"));
    assert!(body.contains("(reference.clone(), document.clone())"));
    assert!(!source.contains("imports.widgets.clone()"));
    assert!(!source.contains("imports.styles.clone()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hh_editor200_preview_import_streaming_clone_release_benchmark() {
    let source = (0..IMPORT_COUNT)
        .map(|index| (format!("preview-import-{index:04}"), document("")))
        .collect::<BTreeMap<_, _>>();

    let mut legacy_result = BTreeMap::new();
    legacy_extend_compiler_imports(&mut legacy_result, &source);
    let mut optimized_result = BTreeMap::new();
    extend_compiler_imports(&mut optimized_result, &source);
    assert_eq!(legacy_result, optimized_result);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                let mut target = BTreeMap::new();
                legacy_extend_compiler_imports(black_box(&mut target), black_box(&source));
                black_box(target);
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                let mut target = BTreeMap::new();
                extend_compiler_imports(black_box(&mut target), black_box(&source));
                black_box(target);
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

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR200_PREVIEW_IMPORT_STREAMING_CLONE_BENCH_V1 \
         import_count={IMPORT_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
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

fn document(id: &str) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Widget,
            id: id.to_string(),
            version: 1,
            display_name: String::new(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: None,
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

fn legacy_extend_compiler_imports(
    target: &mut BTreeMap<String, UiAssetDocument>,
    source: &BTreeMap<String, UiAssetDocument>,
) {
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
