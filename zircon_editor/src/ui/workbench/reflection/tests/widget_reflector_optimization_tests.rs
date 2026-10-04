use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiTreeId};

use super::*;

const VISIT_COUNT: usize = 65_536;
const UNIQUE_NODE_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn node_visits() -> Vec<UiNodeId> {
    (0..VISIT_COUNT)
        .map(|index| UiNodeId::new(((index * 4_099) % UNIQUE_NODE_COUNT) as u64))
        .collect()
}

fn ordered_visit_count(visits: &[UiNodeId]) -> usize {
    let mut visited = BTreeSet::new();
    visits
        .iter()
        .filter(|node_id| visited.insert(**node_id))
        .count()
}

fn hash_visit_count(visits: &[UiNodeId]) -> usize {
    let mut visited = HashSet::with_capacity(UNIQUE_NODE_COUNT);
    visits
        .iter()
        .filter(|node_id| visited.insert(**node_id))
        .count()
}

fn unreserved_rows(visits: &[UiNodeId]) -> Vec<UiNodeId> {
    let mut rows = Vec::new();
    let mut visited = HashSet::new();
    for node_id in visits {
        if visited.insert(*node_id) {
            rows.push(*node_id);
        }
    }
    rows
}

fn reserved_rows(visits: &[UiNodeId]) -> Vec<UiNodeId> {
    let mut rows = Vec::with_capacity(UNIQUE_NODE_COUNT);
    let mut visited = HashSet::with_capacity(UNIQUE_NODE_COUNT);
    for node_id in visits {
        if visited.insert(*node_id) {
            rows.push(*node_id);
        }
    }
    rows
}

fn node(id: u64, children: Vec<UiNodeId>) -> UiReflectorNode {
    let mut node = UiReflectorNode::new(
        UiNodeId::new(id),
        UiNodePath::new(format!("root/{id}")),
        "Panel",
        format!("Node {id}"),
    );
    node.children = children;
    node
}

#[test]
fn optimization_batch_20260826w_editor25_hash_visited_preserves_tree_and_orphan_order() {
    let snapshot = UiReflectorSnapshot::new(
        UiTreeId::new("editor.reflector.optimization"),
        vec![UiNodeId::new(30), UiNodeId::new(30)],
        vec![
            node(30, vec![UiNodeId::new(10)]),
            node(10, vec![UiNodeId::new(20)]),
            node(20, vec![UiNodeId::new(30)]),
            node(5, Vec::new()),
        ],
    );

    let rows = WorkbenchWidgetReflectorModel::new(snapshot).rows();
    assert_eq!(
        rows.iter().map(|row| row.node_id).collect::<Vec<_>>(),
        vec![
            UiNodeId::new(30),
            UiNodeId::new(10),
            UiNodeId::new(20),
            UiNodeId::new(5),
        ]
    );
    assert_eq!(
        rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
        vec![0, 1, 2, 0]
    );
}

#[test]
fn optimization_batch_20260826w_editor25_widget_reflector_uses_hash_visited_membership() {
    let source = include_str!("../widget_reflector.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(
        production.contains("let mut visited = HashSet::with_capacity(self.snapshot.nodes.len());")
    );
    assert!(production.contains("visited: &mut HashSet<UiNodeId>"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826w_editor25_widget_reflector_hash_visited_performance_evidence() {
    let visits = node_visits();
    assert_eq!(ordered_visit_count(&visits), hash_visit_count(&visits));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_visit_count(black_box(&visits)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_visit_count(black_box(&visits)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_visit_count(black_box(&visits)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_visit_count(black_box(&visits)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR25_WIDGET_REFLECTOR_HASH_VISITED_BENCH_V1 visits={VISIT_COUNT} \
             unique_nodes={UNIQUE_NODE_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-visited P95 {:?} exceeded 60% of ordered-visited P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

#[test]
fn optimization_batch_il_editor622_widget_reflector_preallocates_rows_and_membership() {
    let source = include_str!("../widget_reflector.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("let mut rows = Vec::with_capacity(self.snapshot.nodes.len());"));
    assert!(
        production.contains("let mut visited = HashSet::with_capacity(self.snapshot.nodes.len());")
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_il_editor622_preallocated_widget_reflector_performance_evidence() {
    let visits = node_visits();
    assert_eq!(unreserved_rows(&visits), reserved_rows(&visits));

    black_box(unreserved_rows(black_box(&visits)));
    black_box(reserved_rows(black_box(&visits)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_rows(black_box(&visits)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_rows(black_box(&visits)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_rows(black_box(&visits)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_rows(black_box(&visits)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "EDITOR622_PREALLOCATED_WIDGET_REFLECTOR_BENCH_V1 \
             visits={VISIT_COUNT} unique_nodes={UNIQUE_NODE_COUNT} \
             unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated reflector P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}
