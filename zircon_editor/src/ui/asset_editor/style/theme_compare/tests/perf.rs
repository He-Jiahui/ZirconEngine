use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const TOKEN_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jd_editor643_borrows_theme_token_values_during_iteration() {
    let source = include_str!("../../theme_compare.rs");
    let comparison = source
        .split("fn compare_imported_against_local")
        .nth(1)
        .expect("imported theme comparison remains present")
        .split("fn compare_local_against_imports")
        .next()
        .expect("imported theme comparison remains bounded");

    assert!(comparison.contains("for (token_name, imported_value) in &imported.tokens"));
    assert!(comparison.contains("for (token_name, local_value) in &local.tokens"));
    assert!(!comparison.contains("imported.tokens.get(token_name)"));
    assert_eq!(
        comparison.matches("local.tokens.get(token_name)").count(),
        1
    );
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jd_editor643_borrowed_theme_token_iteration_benchmark() {
    let imported = (0..TOKEN_COUNT)
        .map(|index| (index, index.wrapping_mul(31)))
        .collect::<BTreeMap<_, _>>();
    let local = (TOKEN_COUNT / 2..TOKEN_COUNT + TOKEN_COUNT / 2)
        .map(|index| (index, index.wrapping_mul(37)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        legacy_checksum(&imported, &local),
        borrowed_checksum(&imported, &local)
    );

    for _ in 0..4 {
        black_box(measure_iteration(&imported, &local, false));
        black_box(measure_iteration(&imported, &local, true));
    }

    let mut redundant_lookup_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            redundant_lookup_samples.push(measure_iteration(&imported, &local, false));
            borrowed_samples.push(measure_iteration(&imported, &local, true));
        } else {
            borrowed_samples.push(measure_iteration(&imported, &local, true));
            redundant_lookup_samples.push(measure_iteration(&imported, &local, false));
        }
    }

    let redundant_lookup_p95 = percentile(&redundant_lookup_samples, 95);
    let borrowed_p95 = percentile(&borrowed_samples, 95);
    let improvement_percent = redundant_lookup_p95
        .saturating_sub(borrowed_p95)
        .saturating_mul(100)
        / redundant_lookup_p95.max(1);
    println!(
        "EDITOR643_BORROWED_THEME_TOKEN_ITERATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} token_count={TOKEN_COUNT} redundant_lookup_ns={} borrowed_ns={} redundant_lookup_p95_ns={redundant_lookup_p95} borrowed_p95_ns={borrowed_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&redundant_lookup_samples),
        csv(&borrowed_samples),
    );
    assert!(borrowed_p95 <= redundant_lookup_p95 * 80 / 100);
}

fn measure_iteration(
    imported: &BTreeMap<usize, usize>,
    local: &BTreeMap<usize, usize>,
    borrowed: bool,
) -> u128 {
    let started = Instant::now();
    let checksum = if borrowed {
        borrowed_checksum(imported, local)
    } else {
        legacy_checksum(imported, local)
    };
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_checksum(imported: &BTreeMap<usize, usize>, local: &BTreeMap<usize, usize>) -> usize {
    let mut checksum = 0usize;
    for token_name in imported.keys() {
        if let (Some(imported_value), local_value) =
            (imported.get(token_name), local.get(token_name))
        {
            checksum = checksum.wrapping_add(*token_name ^ *imported_value);
            if let Some(local_value) = local_value {
                checksum = checksum.wrapping_add(*local_value);
            }
        }
    }
    for token_name in local.keys() {
        if imported.contains_key(token_name) {
            continue;
        }
        if let Some(local_value) = local.get(token_name) {
            checksum = checksum.wrapping_add(*token_name ^ *local_value);
        }
    }
    checksum
}

fn borrowed_checksum(imported: &BTreeMap<usize, usize>, local: &BTreeMap<usize, usize>) -> usize {
    let mut checksum = 0usize;
    for (token_name, imported_value) in imported {
        checksum = checksum.wrapping_add(*token_name ^ *imported_value);
        if let Some(local_value) = local.get(token_name) {
            checksum = checksum.wrapping_add(*local_value);
        }
    }
    for (token_name, local_value) in local {
        if imported.contains_key(token_name) {
            continue;
        }
        checksum = checksum.wrapping_add(*token_name ^ *local_value);
    }
    checksum
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
