use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ADMISSION_COUNT: usize = 65_536;
const UNIQUE_INSTANCE_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn instance_ids() -> Vec<ViewInstanceId> {
    (0..ADMISSION_COUNT)
        .map(|index| {
            ViewInstanceId::new(format!(
                "editor.builtin.shell.instance.{:05}",
                (index * 4_099) % UNIQUE_INSTANCE_COUNT
            ))
        })
        .collect()
}

fn ordered_admission_count(instance_ids: &[ViewInstanceId]) -> usize {
    let mut present: BTreeSet<ViewInstanceId> = BTreeSet::new();
    let mut admitted = 0;
    for instance_id in instance_ids {
        if present.insert(instance_id.clone()) {
            admitted += 1;
        }
    }
    admitted
}

fn hash_admission_count(instance_ids: &[ViewInstanceId]) -> usize {
    let mut present = HashSet::new();
    let mut admitted = 0;
    for instance_id in instance_ids {
        if admit_present_instance(&mut present, instance_id) {
            admitted += 1;
        }
    }
    admitted
}

#[test]
fn optimization_batch_20260826x_editor13_shell_repair_hash_admission_preserves_first_seen_order() {
    let instance_ids = [
        ViewInstanceId::new("editor.b"),
        ViewInstanceId::new("editor.a"),
        ViewInstanceId::new("editor.b"),
        ViewInstanceId::new("editor.c"),
    ];
    let mut present = HashSet::new();
    let admitted = instance_ids
        .iter()
        .filter(|instance_id| admit_present_instance(&mut present, instance_id))
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(
        admitted,
        vec![
            ViewInstanceId::new("editor.b"),
            ViewInstanceId::new("editor.a"),
            ViewInstanceId::new("editor.c"),
        ]
    );
}

#[test]
fn optimization_batch_20260826x_editor13_shell_repair_uses_borrowed_hash_admission() {
    let source = include_str!("../repair_builtin_shell_layout.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("let mut present: HashSet<_>"));
    assert!(production.contains("present: &mut HashSet<ViewInstanceId>"));
    assert!(production.contains("present.contains(instance_id)"));
    assert!(production.contains("admit_present_instance(&mut present, &repaired_id)"));
    assert!(production.contains("admit_present_instance(present, &repaired_id)"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826x_editor13_shell_repair_hash_admission_performance_evidence() {
    let instance_ids = instance_ids();
    assert_eq!(
        ordered_admission_count(&instance_ids),
        hash_admission_count(&instance_ids)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_admission_count(black_box(&instance_ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_admission_count(black_box(&instance_ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_admission_count(black_box(&instance_ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_admission_count(black_box(&instance_ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR13_SHELL_REPAIR_HASH_ADMISSION_BENCH_V1 admissions={ADMISSION_COUNT} \
             unique_instances={UNIQUE_INSTANCE_COUNT} ordered_set_clones={ADMISSION_COUNT} \
             hash_set_clones={UNIQUE_INSTANCE_COUNT} ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-admission P95 {:?} exceeded 60% of ordered-admission P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
