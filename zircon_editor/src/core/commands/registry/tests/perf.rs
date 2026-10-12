use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::commands::EditorKeymap;

fn legacy_missing_default_keymap_bindings<'a>(
    registry: &'a EditorCommandRegistry,
    keymap: &'a EditorKeymap,
) -> Vec<&'a str> {
    let keymap_commands = keymap
        .bindings()
        .iter()
        .map(|binding| binding.command_id())
        .collect::<BTreeSet<_>>();
    registry
        .commands()
        .filter(|descriptor| descriptor.default_chord().is_some())
        .map(|descriptor| descriptor.id().as_str())
        .filter(|id| !keymap_commands.contains(id))
        .collect()
}

fn benchmark_ids(count: usize, stride: usize) -> Vec<String> {
    (0..count)
        .step_by(stride)
        .map(|index| {
            format!(
                "editor.performance.command.shared.namespace.{}.{index:05}",
                "long-common-prefix".repeat(3)
            )
        })
        .collect()
}

fn btree_missing_count(command_ids: &[String], keymap_ids: &[String]) -> usize {
    let keymap_commands = keymap_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    command_ids
        .iter()
        .filter(|id| !keymap_commands.contains(id.as_str()))
        .count()
}

fn hash_missing_count(command_ids: &[String], keymap_ids: &[String]) -> usize {
    let mut keymap_commands = HashSet::with_capacity(keymap_ids.len());
    keymap_commands.extend(keymap_ids.iter().map(String::as_str));
    command_ids
        .iter()
        .filter(|id| !keymap_commands.contains(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_hv_editor604_hash_membership_preserves_missing_binding_order() {
    let registry = EditorCommandRegistry::default_workbench();
    let keymap = EditorKeymap::default_workbench();

    assert_eq!(
        registry.missing_default_keymap_bindings(&keymap),
        legacy_missing_default_keymap_bindings(&registry, &keymap)
    );
}

#[test]
fn optimization_batch_hv_editor604_missing_binding_lookup_uses_preallocated_hash_set() {
    let source = include_str!("../../registry.rs");
    let body = source
        .split("pub fn missing_default_keymap_bindings")
        .nth(1)
        .expect("missing keymap binding query")
        .split("fn advance_generation")
        .next()
        .expect("bounded missing keymap binding query");

    assert!(body.contains("HashSet::with_capacity(keymap.bindings().len())"));
    assert!(body.contains("keymap_commands.insert(binding.command_id())"));
    assert!(!body.contains("collect::<BTreeSet<_>>()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hv_editor604_keymap_hash_membership_performance_evidence() {
    const COMMAND_COUNT: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;
    let command_ids = benchmark_ids(COMMAND_COUNT, 1);
    let keymap_ids = benchmark_ids(COMMAND_COUNT, 2);
    let measure_btree = || {
        let started = Instant::now();
        black_box(btree_missing_count(
            black_box(&command_ids),
            black_box(&keymap_ids),
        ));
        started.elapsed().as_nanos().max(1)
    };
    let measure_hash = || {
        let started = Instant::now();
        black_box(hash_missing_count(
            black_box(&command_ids),
            black_box(&keymap_ids),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_btree());
        black_box(measure_hash());
    }

    let mut btree_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            btree_samples.push(measure_btree());
            hash_samples.push(measure_hash());
        } else {
            hash_samples.push(measure_hash());
            btree_samples.push(measure_btree());
        }
    }
    btree_samples.sort_unstable();
    hash_samples.sort_unstable();
    let btree_p50 = btree_samples[8];
    let btree_p95 = btree_samples[16];
    let hash_p50 = hash_samples[8];
    let hash_p95 = hash_samples[16];
    println!(
        "EDITOR604_KEYMAP_HASH_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_btree_even btree_first_pairs=9 hash_first_pairs=8 commands={COMMAND_COUNT} keymap_bindings={} btree_p50_ns={btree_p50} btree_p95_ns={btree_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} target_ratio_bp=5000",
        keymap_ids.len(),
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= btree_p95.saturating_mul(5_000),
        "hash membership P95 {hash_p95} ns exceeded 50% of BTree {btree_p95} ns"
    );
}
