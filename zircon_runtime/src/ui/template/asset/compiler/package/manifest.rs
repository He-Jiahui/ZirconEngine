use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetFingerprint, UiCompileCacheKey, UiCompiledAssetDependency,
    UiCompiledAssetDependencyManifest, UiLocalizationDependency, UiResourceDependency,
};

pub(super) fn compiled_asset_dependency_manifest_from_imports(
    _document: &UiAssetDocument,
    cache_key: &UiCompileCacheKey,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
    resource_dependencies: &[UiResourceDependency],
    localization_dependencies: &[UiLocalizationDependency],
) -> UiCompiledAssetDependencyManifest {
    UiCompiledAssetDependencyManifest {
        widget_imports: dependency_entries(widget_imports, &cache_key.widget_imports),
        style_imports: dependency_entries(style_imports, &cache_key.style_imports),
        resource_dependencies: resource_dependencies.to_vec(),
        localization_dependencies: localization_dependencies.to_vec(),
    }
}

fn dependency_entries(
    imports: &BTreeMap<String, UiAssetDocument>,
    fingerprints: &BTreeMap<String, UiAssetFingerprint>,
) -> Vec<UiCompiledAssetDependency> {
    let mut dependencies = Vec::with_capacity(imports.len().min(fingerprints.len()));
    for (reference, document) in imports {
        let Some(fingerprint) = fingerprints.get(reference) else {
            continue;
        };
        dependencies.push(UiCompiledAssetDependency {
            reference: reference.clone(),
            asset_id: document.asset.id.clone(),
            asset_kind: document.asset.kind,
            source_schema_version: document.asset.version,
            fingerprint: *fingerprint,
        });
    }
    dependencies
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[derive(Clone, Copy)]
    struct DependencyRow([u64; 16]);

    #[test]
    fn optimization_batch_ef_dependency_entries_reserve_matching_upper_bound() {
        let source = include_str!("manifest.rs");
        let implementation = source
            .split("fn dependency_entries")
            .nth(1)
            .expect("dependency entry implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("dependency entry production implementation");

        assert!(implementation.contains("imports.len().min(fingerprints.len())"));
        assert!(implementation.contains("Vec::with_capacity"));
        assert!(implementation.contains("dependencies.push("));
        assert!(!implementation.contains(".filter_map("));
    }

    #[test]
    #[ignore = "release-only dependency manifest capacity benchmark"]
    fn optimization_batch_ef_dependency_manifest_capacity_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const ROWS: usize = 2_048;
        const BUILDS_PER_SAMPLE: usize = 64;

        fn measure_legacy(rows: &[DependencyRow]) -> u128 {
            let started = Instant::now();
            let mut checksum = 0u64;
            for _ in 0..BUILDS_PER_SAMPLE {
                let projected = rows
                    .iter()
                    .filter_map(|row| black_box(true).then_some(*row))
                    .collect::<Vec<_>>();
                checksum = checksum.wrapping_add(projected[ROWS - 1].0[0]);
                black_box(projected);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(rows: &[DependencyRow]) -> u128 {
            let started = Instant::now();
            let mut checksum = 0u64;
            for _ in 0..BUILDS_PER_SAMPLE {
                let mut projected = Vec::with_capacity(rows.len());
                for row in rows {
                    if black_box(true) {
                        projected.push(*row);
                    }
                }
                checksum = checksum.wrapping_add(projected[ROWS - 1].0[0]);
                black_box(projected);
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

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let rows = (0..ROWS)
            .map(|index| DependencyRow([index as u64; 16]))
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&rows));
                optimized_samples.push(measure_optimized(&rows));
            } else {
                optimized_samples.push(measure_optimized(&rows));
                legacy_samples.push(measure_legacy(&rows));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME440_DEPENDENCY_MANIFEST_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             rows={ROWS} row_bytes={} builds_per_sample={BUILDS_PER_SAMPLE} \
             pair_order=alternating_legacy_even legacy_capacity=0 optimized_capacity={ROWS} \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            std::mem::size_of::<DependencyRow>(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
            "dependency manifest capacity must reduce P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
