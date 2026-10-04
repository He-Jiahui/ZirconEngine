use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use super::*;
use crate::asset::{AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter};

const BENCHMARK_IMPORTERS: usize = 96;
const BENCHMARK_ITERATIONS: usize = 64;
const BENCHMARK_SAMPLE_PAIRS: usize = 21;
const BENCHMARK_THRESHOLD_PERCENT: u128 = 90;

fn fixture_importer(index: usize) -> Arc<dyn AssetImporterHandler> {
    Arc::new(DiagnosticOnlyAssetImporter::new(
        AssetImporterDescriptor::new(
            format!("plugins07.cached_registry.{index}"),
            "plugins07.cached_registry",
            AssetKind::Data,
            1,
        )
        .with_priority(500)
        .with_source_extensions([format!("p7cache{index}")]),
        "performance fixture",
    ))
}

fn legacy_active_importer_registry(
    plugin_importers: &[Arc<dyn AssetImporterHandler>],
) -> AssetImporterRegistry {
    let mut registry = AssetImporter::default().registry().clone();
    for importer in plugin_importers {
        registry.register_arc(importer.clone()).unwrap();
    }
    registry
}

fn measure_registry_acquisition(
    iterations: usize,
    source_path: &Path,
    mut acquire: impl FnMut() -> AssetImporterRegistry,
) -> u128 {
    let timer = Instant::now();
    let mut checksum = 0_i64;
    for _ in 0..iterations {
        let registry = black_box(acquire());
        checksum += i64::from(
            registry
                .select(black_box(source_path))
                .unwrap()
                .descriptor()
                .priority,
        );
    }
    black_box(checksum);
    timer.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 - 1) / 100]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn cached_active_importer_registry_owns_default_generation() {
    let manager = ProjectAssetManager::default();
    let expected = AssetImporter::default().registry().descriptors();

    assert!(
        !expected.is_empty(),
        "default importer registry is nonempty"
    );
    assert_eq!(manager.importer_registry_read().descriptors(), expected);
}

#[test]
fn cached_active_importer_registry_rejects_conflict_without_mutation() {
    let manager = ProjectAssetManager::default();
    let before = manager.importer_registry_read().descriptors();
    let duplicate = before
        .first()
        .expect("default registry is nonempty")
        .clone();

    let error = manager
        .register_asset_importer(DiagnosticOnlyAssetImporter::new(
            duplicate,
            "duplicate fixture",
        ))
        .expect_err("duplicate default importer must be rejected");

    assert!(error.to_string().contains("already registered"));
    assert_eq!(manager.importer_registry_read().descriptors(), before);
}

#[test]
#[ignore = "release-only performance evidence"]
fn benchmark_cached_active_importer_registry_acquisition() {
    let manager = ProjectAssetManager::default();
    let plugin_importers = (0..BENCHMARK_IMPORTERS)
        .map(fixture_importer)
        .collect::<Vec<_>>();
    for importer in &plugin_importers {
        manager
            .register_asset_importer_arc(importer.clone())
            .unwrap();
    }
    let source_path = Path::new("fixture.p7cache95");
    assert_eq!(
        legacy_active_importer_registry(&plugin_importers)
            .select(source_path)
            .unwrap()
            .descriptor()
            .id,
        manager
            .active_importer_registry()
            .select(source_path)
            .unwrap()
            .descriptor()
            .id
    );

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_registry_acquisition(
                BENCHMARK_ITERATIONS,
                source_path,
                || legacy_active_importer_registry(&plugin_importers),
            ));
            optimized_samples.push(measure_registry_acquisition(
                BENCHMARK_ITERATIONS,
                source_path,
                || manager.active_importer_registry(),
            ));
        } else {
            optimized_samples.push(measure_registry_acquisition(
                BENCHMARK_ITERATIONS,
                source_path,
                || manager.active_importer_registry(),
            ));
            legacy_samples.push(measure_registry_acquisition(
                BENCHMARK_ITERATIONS,
                source_path,
                || legacy_active_importer_registry(&plugin_importers),
            ));
        }
    }

    let legacy_raw = legacy_samples.clone();
    let optimized_raw = optimized_samples.clone();
    let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);

    println!(
        "PERF_RESULT plugins07_cached_active_importer_registry importers={} iterations_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_registry_rebuilds_per_sample={} optimized_registry_rebuilds_per_sample=0 legacy_plugin_registrations_per_sample={} optimized_plugin_registrations_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
        BENCHMARK_IMPORTERS,
        BENCHMARK_ITERATIONS,
        BENCHMARK_SAMPLE_PAIRS,
        BENCHMARK_ITERATIONS,
        BENCHMARK_IMPORTERS * BENCHMARK_ITERATIONS,
        legacy_p95_ns,
        optimized_p95_ns,
        improvement_percent,
        BENCHMARK_THRESHOLD_PERCENT,
        sample_csv(&legacy_raw),
        sample_csv(&optimized_raw),
    );

    assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
    assert_eq!(BENCHMARK_SAMPLE_PAIRS, optimized_raw.len());
    assert!(
        improvement_percent >= BENCHMARK_THRESHOLD_PERCENT,
        "cached active importer registry P95 improvement {improvement_percent}% misses {BENCHMARK_THRESHOLD_PERCENT}% gate"
    );
}
