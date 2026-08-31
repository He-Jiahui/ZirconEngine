use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use zircon_runtime_interface::ui::template::UiChildMount;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ak_component_tree_hash_index_preserves_targets() {
    let root = wide_tree(4);
    let index = ComponentTreeIndex::new(&root);

    assert!(index.node_ids.contains("node-00003"));
    assert!(index.control_ids.contains("control-00003"));
    assert_eq!(
        index.node_control_ids.get("node-00003").copied(),
        Some("control-00003")
    );
    assert_eq!(
        index.target_ref("control-00003").control_id.as_deref(),
        Some("control-00003")
    );
    assert_eq!(
        index.target_ref("missing").node_id.as_deref(),
        Some("missing")
    );
}

#[test]
fn optimization_batch_20260826ak_component_contract_uses_borrowed_hash_indexes() {
    let source = include_str!("../validation.rs");

    assert!(source.contains("node_ids: HashSet<&'a str>"));
    assert!(source.contains("control_ids: HashSet<&'a str>"));
    assert!(source.contains("node_control_ids: HashMap<&'a str, &'a str>"));
    assert!(source.contains("private_targets: HashSet<&'a str>"));
    assert!(!source.contains("node_ids: BTreeSet<&'a str>"));
}

#[test]
#[ignore = "release-only performance contract"]
fn optimization_batch_20260826ak_component_contract_hash_index_p95() {
    let root = wide_tree(4_096);
    let (build_baseline, build_optimized) = paired_samples(
        || {
            black_box(LegacyComponentTreeIndex::new(black_box(&root)));
        },
        || {
            black_box(ComponentTreeIndex::new(black_box(&root)));
        },
    );

    let legacy = LegacyComponentTreeIndex::new(&root);
    let optimized = ComponentTreeIndex::new(&root);
    let probes = (0..4_096)
        .rev()
        .flat_map(|index| {
            [
                format!("node-{index:05}"),
                format!("control-{index:05}"),
                format!("missing-{index:05}"),
            ]
        })
        .collect::<Vec<_>>();
    let (lookup_baseline, lookup_optimized) = paired_samples(
        || {
            black_box(legacy_lookup_checksum(
                black_box(&legacy),
                black_box(&probes),
            ));
        },
        || {
            black_box(hash_lookup_checksum(
                black_box(&optimized),
                black_box(&probes),
            ));
        },
    );

    let build_baseline_p95 = percentile_95(&build_baseline);
    let build_optimized_p95 = percentile_95(&build_optimized);
    let lookup_baseline_p95 = percentile_95(&lookup_baseline);
    let lookup_optimized_p95 = percentile_95(&lookup_optimized);

    println!(
        "RUNTIME74_COMPONENT_CONTRACT_HASH_INDEX_BENCH_V1 \
         build_baseline_p95_ns={} build_optimized_p95_ns={} \
         lookup_baseline_p95_ns={} lookup_optimized_p95_ns={}",
        build_baseline_p95.as_nanos(),
        build_optimized_p95.as_nanos(),
        lookup_baseline_p95.as_nanos(),
        lookup_optimized_p95.as_nanos(),
    );
    assert_at_most_sixty_percent("tree index build", build_baseline_p95, build_optimized_p95);
    assert_at_most_sixty_percent(
        "tree index lookup",
        lookup_baseline_p95,
        lookup_optimized_p95,
    );
}

fn wide_tree(child_count: usize) -> UiNodeDefinition {
    UiNodeDefinition {
        node_id: "root".to_string(),
        control_id: Some("root-control".to_string()),
        children: (0..child_count)
            .map(|index| UiChildMount {
                node: UiNodeDefinition {
                    node_id: format!("node-{index:05}"),
                    control_id: Some(format!("control-{index:05}")),
                    ..UiNodeDefinition::default()
                },
                ..UiChildMount::default()
            })
            .collect(),
        ..UiNodeDefinition::default()
    }
}

struct LegacyComponentTreeIndex<'a> {
    node_ids: BTreeSet<&'a str>,
    control_ids: BTreeSet<&'a str>,
    node_control_ids: BTreeMap<&'a str, &'a str>,
}

impl<'a> LegacyComponentTreeIndex<'a> {
    fn new(root: &'a UiNodeDefinition) -> Self {
        let mut index = Self {
            node_ids: BTreeSet::new(),
            control_ids: BTreeSet::new(),
            node_control_ids: BTreeMap::new(),
        };
        index.visit(root);
        index
    }

    fn visit(&mut self, node: &'a UiNodeDefinition) {
        let _ = self.node_ids.insert(node.node_id.as_str());
        if let Some(control_id) = &node.control_id {
            let _ = self.control_ids.insert(control_id.as_str());
            let _ = self
                .node_control_ids
                .insert(node.node_id.as_str(), control_id.as_str());
        }
        for child in &node.children {
            self.visit(&child.node);
        }
    }
}

fn legacy_lookup_checksum(index: &LegacyComponentTreeIndex<'_>, probes: &[String]) -> usize {
    probes.iter().fold(0, |checksum, probe| {
        checksum
            + usize::from(index.node_ids.contains(probe.as_str()))
            + usize::from(index.control_ids.contains(probe.as_str()))
            + usize::from(index.node_control_ids.contains_key(probe.as_str()))
    })
}

fn hash_lookup_checksum(index: &ComponentTreeIndex<'_>, probes: &[String]) -> usize {
    probes.iter().fold(0, |checksum, probe| {
        checksum
            + usize::from(index.node_ids.contains(probe.as_str()))
            + usize::from(index.control_ids.contains(probe.as_str()))
            + usize::from(index.node_control_ids.contains_key(probe.as_str()))
    })
}

fn paired_samples(
    mut baseline: impl FnMut(),
    mut optimized: impl FnMut(),
) -> (Vec<Duration>, Vec<Duration>) {
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&mut baseline));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            baseline_samples.push(measure(&mut baseline));
        }
    }
    (baseline_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut()) -> Duration {
    let started = Instant::now();
    operation();
    started.elapsed()
}

fn percentile_95(samples: &[Duration]) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn assert_at_most_sixty_percent(label: &str, baseline: Duration, optimized: Duration) {
    assert!(
        optimized.as_nanos().saturating_mul(100) <= baseline.as_nanos().saturating_mul(60),
        "{label}: optimized P95 {optimized:?} exceeded 60% of baseline P95 {baseline:?}",
    );
}
