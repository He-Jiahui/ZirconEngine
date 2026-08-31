use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const VALIDATIONS_PER_SAMPLE: usize = 64;
const LABELS_PER_VALIDATION: usize = 4_096;

#[test]
fn optimization_batch_20260826gm_runtime233_borrowed_labels_preserve_duplicate_detection() {
    let labels = ["mesh", "material", "mesh", "animation", "material"];
    let mut seen = HashSet::with_capacity(labels.len());
    let mut duplicates = Vec::new();

    for label in labels {
        if !seen.insert(label) {
            duplicates.push(label);
        }
    }

    assert_eq!(duplicates, ["mesh", "material"]);
    assert_eq!(seen.len(), 3);
}

#[test]
fn optimization_batch_20260826gm_runtime233_import_validation_borrows_preallocated_labels() {
    let source = include_str!("../metadata.rs");

    assert!(source.contains("HashSet::with_capacity(outcome.entries.len())"));
    assert!(source.contains("if !labels.insert(label)"));
    assert!(!source.contains("let mut labels = HashSet::new();"));
    assert!(!source.contains("labels.insert(label.to_string())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gm_runtime233_import_label_borrowed_set_bench() {
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
        "RUNTIME233_IMPORT_LABEL_BORROWED_SET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
validations_per_sample={VALIDATIONS_PER_SAMPLE} labels_per_validation={LABELS_PER_VALIDATION} \
legacy_owned_label_clones_per_validation={LABELS_PER_VALIDATION} \
optimized_owned_label_clones_per_validation=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(borrow_labels: bool) -> u128 {
    let labels = (0..LABELS_PER_VALIDATION)
        .map(|index| format!("imported-label-{index:04}"))
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut checksum = 0usize;
    for validation in 0..VALIDATIONS_PER_SAMPLE {
        if borrow_labels {
            let mut seen = HashSet::with_capacity(labels.len());
            for label in &labels {
                seen.insert(black_box(label.as_str()));
            }
            checksum ^= black_box(seen.len() ^ seen.capacity() ^ validation);
            black_box(seen);
        } else {
            let mut seen = HashSet::new();
            for label in &labels {
                seen.insert(black_box(label.clone()));
            }
            checksum ^= black_box(seen.len() ^ seen.capacity() ^ validation);
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
