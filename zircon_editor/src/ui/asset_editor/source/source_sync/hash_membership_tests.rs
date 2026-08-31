use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const NODE_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_NODE_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn node_ids() -> Vec<String> {
    (0..NODE_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "widget.generated.outline.node.with.long.identity.{:05}",
                (index * 4_099) % UNIQUE_NODE_COUNT
            )
        })
        .collect()
}

fn ordered_outline_membership(ids: &[String]) -> usize {
    let caller_ids = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let node_ids = caller_ids.into_iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    ids.iter()
        .filter(|id| node_ids.contains(id.as_str()) && seen.insert(id.as_str()))
        .count()
}

fn hash_outline_membership(ids: &[String]) -> usize {
    let node_ids = ids.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut seen = HashSet::with_capacity(node_ids.len());
    ids.iter()
        .filter(|id| node_ids.contains(id.as_str()) && seen.insert(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_20260826af_editor23_hash_outline_membership_preserves_source_order() {
    let source = "[nodes.zeta]\nkind = \"label\"\n[nodes.alpha]\nkind = \"label\"\n[nodes.middle]\nkind = \"label\"\n";
    let index =
        build_source_outline_index_for_node_ids(source, ["middle", "zeta", "alpha", "zeta"]);

    assert_eq!(
        index
            .entries()
            .iter()
            .map(|entry| entry.node_id.as_str())
            .collect::<Vec<_>>(),
        ["zeta", "alpha", "middle"]
    );
    assert_eq!(index.index_for_node("zeta"), Some(0));
    assert_eq!(index.index_for_node("middle"), Some(2));
}

#[test]
fn optimization_batch_20260826af_editor23_source_outline_uses_single_hash_membership_pass() {
    let source = include_str!("../source_sync.rs");
    let caller = source
        .split("pub(crate) fn build_source_outline_index(")
        .nth(1)
        .and_then(|body| {
            body.split("fn build_source_outline_index_for_node_ids")
                .next()
        })
        .expect("outline entry point");
    let builder = source
        .split("fn build_source_outline_index_for_node_ids")
        .nth(1)
        .and_then(|body| body.split("fn direct_outline_entry").next())
        .expect("outline builder");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashSet};"));
    assert!(!caller.contains("collect::<BTreeSet"));
    assert!(builder.contains("collect::<HashSet<_>>()"));
    assert!(builder.contains("HashSet::with_capacity(node_ids.len())"));
    assert!(!builder.contains("let mut seen_tree_node_ids = BTreeSet"));
    assert!(builder.contains("entries.sort_by"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826af_editor23_source_outline_hash_membership_performance_evidence() {
    let ids = node_ids();
    assert_eq!(
        ordered_outline_membership(&ids),
        hash_outline_membership(&ids)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_outline_membership(black_box(&ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_outline_membership(black_box(&ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_outline_membership(black_box(&ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_outline_membership(black_box(&ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR23_SOURCE_OUTLINE_HASH_MEMBERSHIP_BENCH_V1 \
         admissions={NODE_ADMISSION_COUNT} unique_nodes={UNIQUE_NODE_COUNT} \
         ordered_membership_builds=3 hash_membership_builds=2 ordered_p95_ns={} hash_p95_ns={}",
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
