use std::hint::black_box;
use std::time::Instant;

use super::*;

const ADMISSION_COMMAND_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

fn descriptor(id: &str) -> EditorCommandDescriptor {
    EditorCommandDescriptor::operation(EditorOperationPath::parse(id).unwrap())
}

fn admission_ids() -> Vec<EditorOperationPath> {
    (0..ADMISSION_COMMAND_COUNT)
        .map(|index| {
            EditorOperationPath::parse(format!("editor.performance.command_{index:05}")).unwrap()
        })
        .collect()
}

fn legacy_admit(ids: &[EditorOperationPath]) -> BTreeSet<EditorOperationPath> {
    let mut admitted = BTreeSet::new();
    for id in ids {
        if !admitted.contains(id) {
            admitted.insert(id.clone());
        }
    }
    admitted
}

fn single_pass_admit(ids: &[EditorOperationPath]) -> BTreeSet<EditorOperationPath> {
    let mut admitted = BTreeSet::new();
    for id in ids {
        admitted.insert(id.clone());
    }
    admitted
}

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826c_editor08_contribution_admission_uses_one_tree_traversal() {
    let source = include_str!("../contribution.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("fn claim_command_id("));
    assert!(source.contains("self.command_ids.insert(command_id.clone())"));
    assert!(!source.contains("self.command_ids.contains("));
}

#[test]
fn optimization_batch_20260826c_editor08_contribution_admission_keeps_seen_ids_after_take() {
    let mut contributions = EditorCommandContributionSet::default();
    contributions
        .register(descriptor("editor.test.unique"))
        .unwrap();
    assert_eq!(contributions.take_pending().len(), 1);

    assert_eq!(
        contributions.register(descriptor("editor.test.unique")),
        Err(EditorCommandRegistryError::DuplicateCommand(
            EditorOperationPath::parse("editor.test.unique").unwrap()
        ))
    );
    assert!(contributions.take_pending().is_empty());
    assert_eq!(contributions.command_ids().count(), 1);
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826c_editor08_contribution_admission_performance_evidence() {
    let ids = admission_ids();

    for _ in 0..3 {
        assert_eq!(black_box(legacy_admit(&ids)).len(), ADMISSION_COMMAND_COUNT);
        assert_eq!(
            black_box(single_pass_admit(&ids)).len(),
            ADMISSION_COMMAND_COUNT
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_micros(|| {
                black_box(legacy_admit(black_box(&ids)));
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                black_box(single_pass_admit(black_box(&ids)));
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "EDITOR08_CONTRIBUTION_SINGLE_PASS_ADMISSION_BENCH_V1 sample_pairs={} command_ids={} legacy_membership_tree_traversals={} optimized_membership_tree_traversals={} legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        SAMPLE_PAIRS,
        ADMISSION_COMMAND_COUNT,
        ADMISSION_COMMAND_COUNT * 2,
        ADMISSION_COMMAND_COUNT,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "single-pass contribution admission p95 must be at least 20% below contains-plus-insert: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}
