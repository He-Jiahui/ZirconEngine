use std::collections::HashSet;

use toml::Value;

use super::attributes::first_string_value;

pub(super) fn command_id_set(value: Option<&Value>) -> HashSet<String> {
    value
        .map(command_id_values)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

pub(super) fn command_id_values(value: &Value) -> Vec<String> {
    match value {
        Value::Array(values) => values
            .iter()
            .flat_map(command_id_values)
            .filter(|value| !value.is_empty())
            .collect(),
        Value::String(value) => vec![value.split('|').next().unwrap_or(value).trim().to_string()],
        Value::Table(values) => {
            first_string_value(values, &["id", "command_id", "commandId", "value", "key"])
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const RECENT_ID_COUNT: usize = 8_192;
    const MEMBERSHIP_LOOKUP_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn recent_ids() -> Vec<String> {
        (0..RECENT_ID_COUNT)
            .map(|index| format!("editor.command.generated.{index:05}"))
            .collect()
    }

    fn membership_lookups(recent_ids: &[String]) -> Vec<String> {
        (0..MEMBERSHIP_LOOKUP_COUNT)
            .map(|index| recent_ids[(index * 4_099) % recent_ids.len()].clone())
            .collect()
    }

    fn ordered_match_count(recent_ids: &[String], lookups: &[String]) -> usize {
        let recent = recent_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        lookups
            .iter()
            .filter(|command_id| recent.contains(command_id.as_str()))
            .count()
    }

    fn hash_match_count(recent_ids: &[String], lookups: &[String]) -> usize {
        let recent = recent_ids
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        lookups
            .iter()
            .filter(|command_id| recent.contains(command_id.as_str()))
            .count()
    }

    #[test]
    fn optimization_batch_20260826y_editor08_recent_command_hash_set_preserves_parsing_and_membership(
    ) {
        let value = Value::Array(vec![
            Value::String("editor.open | Open".to_string()),
            Value::String("editor.save|Save".to_string()),
            Value::String("editor.open|Duplicate".to_string()),
            Value::String(String::new()),
        ]);

        let recent = command_id_set(Some(&value));
        assert_eq!(recent.len(), 2);
        assert!(recent.contains("editor.open"));
        assert!(recent.contains("editor.save"));
        assert!(!recent.contains("editor.close"));
    }

    #[test]
    fn optimization_batch_20260826y_editor08_recent_command_ids_use_hash_membership() {
        let source = include_str!("ids.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("-> HashSet<String>"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826y_editor08_recent_command_hash_membership_performance_evidence() {
        let recent_ids = recent_ids();
        let lookups = membership_lookups(&recent_ids);
        assert_eq!(
            ordered_match_count(&recent_ids, &lookups),
            hash_match_count(&recent_ids, &lookups)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_match_count(
                    black_box(&recent_ids),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_match_count(
                    black_box(&recent_ids),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_match_count(
                    black_box(&recent_ids),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_match_count(
                    black_box(&recent_ids),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR08_RECENT_COMMAND_HASH_MEMBERSHIP_BENCH_V1 recent_ids={RECENT_ID_COUNT} \
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
}
