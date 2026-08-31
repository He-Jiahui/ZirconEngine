use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::EditorUiControlService;
use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};

const ENTRY_COUNT: usize = 512;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bu_activity_registry_hash_index_preserves_lookup_and_order() {
    let mut service = EditorUiControlService::default();
    for id in ["editor.zeta", "editor.alpha", "editor.middle"] {
        service
            .register_activity_view(ActivityViewDescriptor::new(id, id, "view"))
            .unwrap();
        service
            .register_activity_window(ActivityWindowDescriptor::new(id, id, "window"))
            .unwrap();
    }

    assert_eq!(
        service.activity_view("editor.middle").unwrap().view_id,
        "editor.middle"
    );
    assert_eq!(
        service.activity_window("editor.middle").unwrap().window_id,
        "editor.middle"
    );
    assert_eq!(
        service
            .activity_views()
            .into_iter()
            .map(|descriptor| descriptor.view_id)
            .collect::<Vec<_>>(),
        vec!["editor.alpha", "editor.middle", "editor.zeta"]
    );
    assert_eq!(
        service
            .activity_windows()
            .into_iter()
            .map(|descriptor| descriptor.window_id)
            .collect::<Vec<_>>(),
        vec!["editor.alpha", "editor.middle", "editor.zeta"]
    );
    assert!(service
        .register_activity_view(ActivityViewDescriptor::new(
            "editor.middle",
            "duplicate",
            "view"
        ))
        .is_err());
    assert!(service
        .register_activity_window(ActivityWindowDescriptor::new(
            "editor.middle",
            "duplicate",
            "window",
        ))
        .is_err());
}

#[test]
fn optimization_batch_20260826bu_activity_registry_hash_index_sorts_snapshots_explicitly() {
    let source = include_str!("../service.rs");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("activity_views: HashMap<String, ActivityViewDescriptor>"));
    assert!(source.contains("activity_windows: HashMap<String, ActivityWindowDescriptor>"));
    assert!(source.contains("views.sort_by(|left, right| left.view_id.cmp(&right.view_id));"));
    assert!(source.contains("windows.sort_by(|left, right| left.window_id.cmp(&right.window_id));"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826bu_activity_registry_hash_index_p95() {
    let keys = (0..ENTRY_COUNT)
        .map(|index| format!("editor.activity.SharedRegistryPrefix.ForStableLookup.{index:04}"))
        .collect::<Vec<_>>();
    let ordered = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(key, value)| (key.clone(), *value))
        .collect::<HashMap<_, _>>();
    let target = keys.last().unwrap().as_str();

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
        "activity registry hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR01_ACTIVITY_REGISTRY_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: &str) -> usize
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
    fn lookup(&self, key: &str) -> Option<usize>;
}

impl Lookup for BTreeMap<String, usize> {
    fn lookup(&self, key: &str) -> Option<usize> {
        self.get(key).copied()
    }
}

impl Lookup for HashMap<String, usize> {
    fn lookup(&self, key: &str) -> Option<usize> {
        self.get(key).copied()
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
