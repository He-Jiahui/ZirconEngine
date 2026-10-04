use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::template::{UiAssetImports, UiComponentDefinition};

use super::*;

const DEPENDENCY_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_DEPENDENCY_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn dependency_names() -> Vec<String> {
    (0..DEPENDENCY_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "GeneratedWidgetDependencyWithLongSharedIdentity{:05}",
                (index * 4_099) % UNIQUE_DEPENDENCY_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(names: &[String]) -> usize {
    let mut visited = BTreeSet::new();
    names
        .iter()
        .filter(|name| visited.insert(name.as_str()))
        .count()
}

fn hash_unique_count(names: &[String]) -> usize {
    let mut visited = HashSet::new();
    names
        .iter()
        .filter(|name| visited.insert(name.as_str()))
        .count()
}

fn component_referencing(node_id: &str, component_name: &str) -> UiComponentDefinition {
    UiComponentDefinition {
        root: UiNodeDefinition {
            node_id: node_id.to_string(),
            kind: UiNodeDefinitionKind::Component,
            component: Some(component_name.to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn optimization_batch_20260826ac_editor23_hash_component_closure_terminates_cycles() {
    let document = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "ui.tests.cyclic_components".to_string(),
            version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            display_name: "Cyclic Components".to_string(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: None,
        components: BTreeMap::from([
            (
                "Alpha".to_string(),
                component_referencing("alpha.root", "Beta"),
            ),
            (
                "Beta".to_string(),
                component_referencing("beta.root", "Alpha"),
            ),
        ]),
        stylesheets: Vec::new(),
    };

    let dependencies = collect_component_dependency_names(&document, "Alpha").unwrap();

    assert_eq!(dependencies.len(), 2);
    assert!(dependencies.contains("Alpha"));
    assert!(dependencies.contains("Beta"));
}

#[test]
fn optimization_batch_20260826ac_editor23_widget_promotion_uses_hash_closure_and_ordered_output() {
    let source = include_str!("../promote_widget.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{HashSet, VecDeque};"));
    assert!(production.contains("-> Option<HashSet<String>>"));
    assert!(production.contains("HashSet::with_capacity(dependency_capacity)"));
    assert!(production.contains("VecDeque::with_capacity(dependency_capacity)"));
    assert!(production.contains("document.components.get(name).cloned()"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ac_editor23_widget_dependency_hash_closure_performance_evidence() {
    let names = dependency_names();
    assert_eq!(ordered_unique_count(&names), hash_unique_count(&names));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&names)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&names)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&names)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&names)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR23_WIDGET_DEPENDENCY_HASH_CLOSURE_BENCH_V1 \
             admissions={DEPENDENCY_ADMISSION_COUNT} unique_dependencies={UNIQUE_DEPENDENCY_COUNT} \
             ordered_component_map=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-closure P95 {:?} exceeded 60% of ordered-closure P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
