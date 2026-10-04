use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::asset::{AssetImportContext, AssetImportOutcome, AssetKind, FunctionAssetImporter};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 20_000;

#[test]
fn registry_label_hotpath_contract_descriptor_selection_borrows_slot() {
    let registry = benchmark_registry();
    let path = Path::new("fixture.plugins07");

    let selected = registry.descriptor_for_source(path).unwrap();

    assert_eq!(selected.id, "plugins07.registry.descriptor");
    assert_eq!(selected.plugin_id, "plugins07.registry");
    assert_eq!(selected.source_extensions, ["plugins07"]);
}

#[test]
#[ignore = "release performance gate"]
fn registry_label_hotpath_performance_release_descriptor_selection() {
    let registry = benchmark_registry();
    let path = Path::new("fixture.plugins07");
    for _ in 0..4 {
        black_box(measure_legacy(&registry, path));
        black_box(measure_borrowed(&registry, path));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (
                measure_legacy(&registry, path),
                measure_borrowed(&registry, path),
            )
        } else {
            let optimized_ns = measure_borrowed(&registry, path);
            (measure_legacy(&registry, path), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_registry_borrowed_descriptor_selection sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=3 legacy_importer_arc_clones_per_sample={LOOKUPS_PER_SAMPLE} optimized_importer_arc_clones_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= 3,
        "borrowed descriptor selection must improve P95 by at least 3%"
    );
}

fn benchmark_registry() -> AssetImporterRegistry {
    let mut registry = AssetImporterRegistry::default();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "plugins07.registry.descriptor",
                "plugins07.registry",
                AssetKind::Data,
                1,
            )
            .with_source_extensions(["plugins07"]),
            unreachable_import,
        ))
        .unwrap();
    registry
}

fn unreachable_import(
    _context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    unreachable!("descriptor selection does not invoke the importer")
}

fn measure_legacy(registry: &AssetImporterRegistry, path: &Path) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let descriptor = black_box(registry)
            .select(black_box(path))
            .unwrap()
            .descriptor()
            .clone();
        black_box(descriptor);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_borrowed(registry: &AssetImporterRegistry, path: &Path) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let descriptor = black_box(registry)
            .descriptor_for_source(black_box(path))
            .unwrap();
        black_box(descriptor);
    }
    started.elapsed().as_nanos().max(1)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
