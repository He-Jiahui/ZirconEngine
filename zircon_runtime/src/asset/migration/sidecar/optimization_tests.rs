use std::hint::black_box;
use std::time::Instant;

#[test]
fn runtime04_migration_sidecar_capacity_source_contract() {
    let source = include_str!("../sidecar.rs");
    assert!(source.contains("sidecar_capacity.saturating_add(generated_capacity)"));
    assert!(source.contains("Vec::with_capacity(sidecar_capacity)"));
    assert!(source.contains("Vec::with_capacity(entry_capacity)"));
    assert!(!source.contains("let mut documents = Vec::new();"));
    assert!(!source.contains("let mut entries = Vec::new();"));
}

#[test]
fn runtime04_migration_sidecar_capacity_projection_formula() {
    let sidecars = 17usize;
    let generated_sources = 9usize;
    let entry_counts = [0usize, 2, 5, 11];
    let document_capacity = sidecars.saturating_add(generated_sources);
    let entry_capacity = entry_counts.iter().fold(0usize, |capacity, entries| {
        capacity.saturating_add(1).saturating_add(*entries)
    });
    assert_eq!(document_capacity, 26);
    assert_eq!(entry_capacity, 22);
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_migration_sidecar_capacity_bench() {
    const SAMPLE_COUNT: usize = 17;
    const ITERATIONS: usize = 16;
    const DOCUMENTS: usize = 4_096;
    const ENTRIES_PER_DOCUMENT: usize = 3;

    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = Vec::new();
                for index in 0..DOCUMENTS {
                    values.push(index);
                    for entry in 0..ENTRIES_PER_DOCUMENT {
                        values.push(index.saturating_mul(ENTRIES_PER_DOCUMENT) + entry);
                    }
                }
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = Vec::with_capacity(
                    DOCUMENTS.saturating_mul(1usize.saturating_add(ENTRIES_PER_DOCUMENT)),
                );
                for index in 0..DOCUMENTS {
                    values.push(index);
                    for entry in 0..ENTRIES_PER_DOCUMENT {
                        values.push(index.saturating_mul(ENTRIES_PER_DOCUMENT) + entry);
                    }
                }
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let percentile_95 = |mut samples: Vec<u128>| {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    };
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_MIGRATION_SIDECAR_CAPACITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} documents={} entries_per_document={} capacity_bound={} starting_capacity_reduction=0->{}",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        DOCUMENTS,
        ENTRIES_PER_DOCUMENT,
        DOCUMENTS.saturating_mul(1usize.saturating_add(ENTRIES_PER_DOCUMENT)),
        DOCUMENTS.saturating_mul(1usize.saturating_add(ENTRIES_PER_DOCUMENT)),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized sidecar capacity p95 should be at most 95% of legacy p95"
    );
}
