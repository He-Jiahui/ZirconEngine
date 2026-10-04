use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const IMPORTED_RULE_COUNT: usize = 8_192;
const RULE_LOOKUP_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn imported_rule_signatures() -> Vec<String> {
    (0..IMPORTED_RULE_COUNT)
        .map(|index| format!("theme.rule.{index:04}|color=token.{index:04}"))
        .collect()
}

fn rule_lookups(imported_signatures: &[String]) -> Vec<String> {
    (0..RULE_LOOKUP_COUNT)
        .map(|index| imported_signatures[(index * 4_099) % imported_signatures.len()].clone())
        .collect()
}

fn ordered_duplicate_count(imported_signatures: &[String], lookups: &[String]) -> usize {
    let imported_rules = imported_signatures.iter().cloned().collect::<BTreeSet<_>>();
    lookups
        .iter()
        .filter(|signature| imported_rules.contains(*signature))
        .count()
}

fn hash_duplicate_count(imported_signatures: &[String], lookups: &[String]) -> usize {
    let imported_rules = imported_signatures.iter().cloned().collect::<HashSet<_>>();
    lookups
        .iter()
        .filter(|signature| imported_rules.contains(*signature))
        .count()
}

#[test]
fn optimization_batch_20260826t_editor23_hash_rule_membership_matches_ordered_membership() {
    let imported_signatures = imported_rule_signatures();
    let lookups = rule_lookups(&imported_signatures);

    assert_eq!(
        ordered_duplicate_count(&imported_signatures, &lookups),
        hash_duplicate_count(&imported_signatures, &lookups)
    );
    assert_eq!(
        hash_duplicate_count(&imported_signatures, &lookups),
        RULE_LOOKUP_COUNT
    );
}

#[test]
fn optimization_batch_20260826t_editor23_theme_projection_uses_hash_membership() {
    let source = include_str!("../action_projection.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashMap, HashSet};"));
    assert!(production.contains("collect::<HashSet<_>>()"));
    assert!(production.contains("HashMap<(&'a str, &'a str)"));
    assert!(production.contains("prefixed_variant_prefix"));
    assert!(!production.contains("BTreeSet"));
    assert!(!production.contains("prefixed_id.to_string() + \"_\""));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826t_editor23_theme_rule_hash_membership_performance_evidence() {
    let imported_signatures = imported_rule_signatures();
    let lookups = rule_lookups(&imported_signatures);
    assert_eq!(
        ordered_duplicate_count(&imported_signatures, &lookups),
        hash_duplicate_count(&imported_signatures, &lookups)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_duplicate_count(
                black_box(&imported_signatures),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_duplicate_count(
                black_box(&imported_signatures),
                black_box(&lookups),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_duplicate_count(
                black_box(&imported_signatures),
                black_box(&lookups),
            ));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_duplicate_count(
                black_box(&imported_signatures),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR23_THEME_PROJECTION_HASH_MEMBERSHIP_BENCH_V1 imported_rules={IMPORTED_RULE_COUNT} \
             lookups={RULE_LOOKUP_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
