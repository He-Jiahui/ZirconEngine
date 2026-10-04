use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const IMPORTED_TOKEN_COUNT: usize = 512;
const LOCAL_TOKEN_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jf_editor645_avoids_prefix_candidate_allocations() {
    let source = include_str!("../../action_projection.rs");
    let renames = source
        .split("pub(super) fn resolve_local_clone_token_renames")
        .nth(1)
        .expect("local clone token rename resolver remains present")
        .split("pub(super) fn find_local_cloned_stylesheet")
        .next()
        .expect("local clone token rename resolver remains bounded");

    assert!(renames.contains(".strip_prefix(&prefixed_base)"));
    assert!(!renames.contains("prefixed_base.clone() + \"_\""));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jf_editor645_prefix_candidate_benchmark() {
    let local_tokens = (0..LOCAL_TOKEN_COUNT)
        .map(|index| (format!("theme_token_{index:05}"), index.wrapping_mul(31)))
        .collect::<BTreeMap<_, _>>();
    let imported_tokens = (0..IMPORTED_TOKEN_COUNT)
        .map(|index| (format!("token_{index:05}"), index))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        legacy_rename_count(&local_tokens, &imported_tokens),
        borrowed_rename_count(&local_tokens, &imported_tokens)
    );

    for _ in 0..4 {
        black_box(measure_renames(&local_tokens, &imported_tokens, false));
        black_box(measure_renames(&local_tokens, &imported_tokens, true));
    }

    let mut concatenating_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            concatenating_samples.push(measure_renames(&local_tokens, &imported_tokens, false));
            borrowed_samples.push(measure_renames(&local_tokens, &imported_tokens, true));
        } else {
            borrowed_samples.push(measure_renames(&local_tokens, &imported_tokens, true));
            concatenating_samples.push(measure_renames(&local_tokens, &imported_tokens, false));
        }
    }

    let concatenating_p95 = percentile(&concatenating_samples, 95);
    let borrowed_p95 = percentile(&borrowed_samples, 95);
    let improvement_percent = concatenating_p95
        .saturating_sub(borrowed_p95)
        .saturating_mul(100)
        / concatenating_p95.max(1);
    println!(
        "EDITOR645_PREFIX_CANDIDATE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} imported_tokens={IMPORTED_TOKEN_COUNT} local_tokens={LOCAL_TOKEN_COUNT} concatenating_ns={} borrowed_ns={} concatenating_p95_ns={concatenating_p95} borrowed_p95_ns={borrowed_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&concatenating_samples),
        csv(&borrowed_samples),
    );
    assert!(borrowed_p95 <= concatenating_p95 * 80 / 100);
}

fn measure_renames(
    local_tokens: &BTreeMap<String, usize>,
    imported_tokens: &BTreeMap<String, usize>,
    borrowed: bool,
) -> u128 {
    let started = Instant::now();
    let count = if borrowed {
        borrowed_rename_count(local_tokens, imported_tokens)
    } else {
        legacy_rename_count(local_tokens, imported_tokens)
    };
    black_box(count);
    started.elapsed().as_nanos().max(1)
}

fn legacy_rename_count(
    local_tokens: &BTreeMap<String, usize>,
    imported_tokens: &BTreeMap<String, usize>,
) -> usize {
    imported_tokens
        .keys()
        .filter(|token_name| {
            let prefixed_base = format!("theme_{token_name}");
            local_tokens.keys().any(|candidate| {
                candidate.as_str() == prefixed_base
                    || candidate.starts_with(&(prefixed_base.clone() + "_"))
            })
        })
        .count()
}

fn borrowed_rename_count(
    local_tokens: &BTreeMap<String, usize>,
    imported_tokens: &BTreeMap<String, usize>,
) -> usize {
    imported_tokens
        .keys()
        .filter(|token_name| {
            let prefixed_base = format!("theme_{token_name}");
            local_tokens.keys().any(|candidate| {
                candidate.as_str() == prefixed_base
                    || candidate
                        .strip_prefix(&prefixed_base)
                        .is_some_and(|suffix| suffix.starts_with('_'))
            })
        })
        .count()
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
