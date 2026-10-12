use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const OPTION_ID_COUNT: usize = 8_192;
const MEMBERSHIP_LOOKUP_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn option_ids() -> Vec<String> {
    (0..OPTION_ID_COUNT)
        .map(|index| format!("editor.pane.option.generated.{index:05}"))
        .collect()
}

fn membership_lookups(option_ids: &[String]) -> Vec<String> {
    (0..MEMBERSHIP_LOOKUP_COUNT)
        .map(|index| option_ids[(index * 4_099) % option_ids.len()].clone())
        .collect()
}

fn ordered_match_count(option_ids: &[String], lookups: &[String]) -> usize {
    let values = option_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    lookups
        .iter()
        .filter(|option_id| values.contains(option_id.as_str()))
        .count()
}

fn hash_match_count(option_ids: &[String], lookups: &[String]) -> usize {
    let values = option_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    lookups
        .iter()
        .filter(|option_id| values.contains(option_id.as_str()))
        .count()
}

#[test]
fn option_query_matching_is_ascii_case_insensitive_without_normalized_row_strings() {
    assert!(contains_ascii_case_insensitive("Open Project", "open pro"));
    assert!(!contains_ascii_case_insensitive("Open Project", "save"));
}

#[test]
fn option_set_matching_checks_id_label_and_raw_keys() {
    let option = structured_option("file.open|label=Open Project,focused");

    assert!(option_matches_set(
        &option,
        &HashSet::from(["Open Project".to_string()])
    ));
    assert!(!option_matches_set(
        &option,
        &HashSet::from(["file.save".to_string()])
    ));
}

#[test]
fn optimization_batch_20260826z_editor01_pane_option_hash_sets_preserve_state_and_input_order() {
    let attributes = BTreeMap::from([
        (
            "selected_options".to_string(),
            toml::Value::Array(vec![toml::Value::String("option.a".to_string())]),
        ),
        (
            "disabled_options".to_string(),
            toml::Value::Array(vec![toml::Value::String("option.b".to_string())]),
        ),
    ]);
    let options = vec![
        "option.b|label=Beta".to_string(),
        "option.a|label=Alpha".to_string(),
    ];

    let projected = structured_options_for_node(&options, &attributes);
    assert_eq!(
        projected
            .iter()
            .map(|option| option.id.as_ref())
            .collect::<Vec<&str>>(),
        vec!["option.b", "option.a"]
    );
    assert!(projected[0].disabled);
    assert!(!projected[0].selected);
    assert!(projected[1].selected);
    assert!(!projected[1].disabled);
}

#[test]
fn optimization_batch_20260826z_editor01_pane_option_projection_uses_hash_membership() {
    let source = include_str!("../pane_option_projection.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains("values: &HashSet<String>"));
    assert!(production.contains("-> HashSet<String>"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826z_editor01_pane_option_hash_membership_performance_evidence() {
    let option_ids = option_ids();
    let lookups = membership_lookups(&option_ids);
    assert_eq!(
        ordered_match_count(&option_ids, &lookups),
        hash_match_count(&option_ids, &lookups)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_match_count(
                black_box(&option_ids),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_match_count(
                black_box(&option_ids),
                black_box(&lookups),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_match_count(
                black_box(&option_ids),
                black_box(&lookups),
            ));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_match_count(
                black_box(&option_ids),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR01_PANE_OPTION_HASH_MEMBERSHIP_BENCH_V1 option_ids={OPTION_ID_COUNT} \
             lookups={MEMBERSHIP_LOOKUP_COUNT} ordered_lookup_class=log_n \
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
