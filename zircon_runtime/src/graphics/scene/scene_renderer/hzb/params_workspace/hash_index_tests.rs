use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::{HzbOcclusionParamsPrepareStats, HzbOcclusionParamsWorkspace};

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn runtime94_hzb_params_hash_workspace_reuses_each_workspace_buffer() {
    let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
        return;
    };
    let mut workspace = HzbOcclusionParamsWorkspace::default();

    let first = workspace.prepare(&backend.device, 17, 64);
    assert!(
        workspace.commit(
            first
                .commit
                .expect("first workspace prepare must publish a commit")
        )
    );
    let repeated = workspace.prepare(&backend.device, 17, 64);
    let second_workspace = workspace.prepare(&backend.device, 23, 64);

    assert!(Arc::ptr_eq(&first.buffer, &repeated.buffer));
    assert!(!Arc::ptr_eq(&first.buffer, &second_workspace.buffer));
    assert_eq!(repeated.stats, HzbOcclusionParamsPrepareStats::default());
    assert_eq!(second_workspace.stats.created_buffer_count, 1);
    assert_eq!(workspace.entries.len(), 2);
}

#[test]
fn runtime94_hzb_params_hash_workspace_has_no_order_contract() {
    let source = include_str!("../params_workspace.rs");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("entries: HashMap<u64, HzbOcclusionParamsEntry>"));
    assert!(!source.contains("BTreeMap"));
    assert!(!source.contains("entries.keys()"));
    assert!(!source.contains("entries.values()"));
    assert!(!source.contains("entries.iter()"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime94_hzb_params_hash_workspace_p95() {
    let ordered = (0..ENTRY_COUNT as u64)
        .map(|workspace_id| (workspace_id, workspace_id as usize + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(workspace_id, value)| (*workspace_id, *value))
        .collect::<HashMap<_, _>>();
    let target = ENTRY_COUNT as u64 - 1;

    let mut ordered_lookup = || repeated_lookup(&ordered, target);
    let mut hash_lookup = || repeated_lookup(&hashed, target);
    assert_eq!(black_box(ordered_lookup()), black_box(hash_lookup()));

    let mut ordered_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            ordered_ns.push(measure_ns(&mut ordered_lookup));
            hash_ns.push(measure_ns(&mut hash_lookup));
        } else {
            hash_ns.push(measure_ns(&mut hash_lookup));
            ordered_ns.push(measure_ns(&mut ordered_lookup));
        }
    }

    let ordered_p50 = nearest_rank(&ordered_ns, 50);
    let ordered_p95 = nearest_rank(&ordered_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= ordered_p95.saturating_mul(7),
        "HZB params workspace hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "RUNTIME94_HZB_PARAMS_HASH_WORKSPACE_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: u64) -> usize
where
    V: Lookup,
{
    let mut total = 0_usize;
    for _ in 0..HIT_COUNT {
        total = total.wrapping_add(black_box(map.lookup(black_box(target))).unwrap_or_default());
    }
    total
}

trait Lookup {
    fn lookup(&self, key: u64) -> Option<usize>;
}

impl Lookup for BTreeMap<u64, usize> {
    fn lookup(&self, key: u64) -> Option<usize> {
        self.get(&key).copied()
    }
}

impl Lookup for HashMap<u64, usize> {
    fn lookup(&self, key: u64) -> Option<usize> {
        self.get(&key).copied()
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
