use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::template::UiBindingRef;

use super::EditorTemplateAdapter;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

const ENTRY_COUNT: usize = 1_024;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bw_template_binding_hash_index_preserves_resolution() {
    let mut adapter = EditorTemplateAdapter::default();
    let binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "OpenProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.project.open"),
    );
    adapter
        .register_binding("WorkbenchMenuBar/OpenProject", binding)
        .unwrap();
    assert!(adapter
        .register_binding(
            "WorkbenchMenuBar/OpenProject",
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "Duplicate",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action("workbench.project.duplicate"),
            ),
        )
        .is_err());

    let resolved = adapter
        .resolve_binding(&binding_ref(
            "WorkbenchMenuBar/OpenProject",
            UiEventKind::Click,
        ))
        .unwrap();
    assert_eq!(resolved.path().view_id, "WorkbenchMenuBar");
    assert_eq!(resolved.path().control_id, "OpenProject");
    assert!(adapter
        .resolve_binding(&binding_ref(
            "WorkbenchMenuBar/OpenProject",
            UiEventKind::Change,
        ))
        .is_err());
}

#[test]
fn optimization_batch_20260826bw_template_binding_hash_index_uses_single_probe_registration() {
    let source = include_str!("../adapter.rs");

    assert!(source.contains("use std::collections::hash_map::Entry;"));
    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("bindings: HashMap<String, EditorUiBinding>"));
    assert!(source.contains("self.bindings.entry(binding_id)"));
    assert!(!source.contains("self.bindings.contains_key"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826bw_template_binding_hash_index_p95() {
    let keys = (0..ENTRY_COUNT)
        .map(|index| format!("Workbench/SharedTemplateBindingPrefix/{index:04}"))
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
        "template binding hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR01_TEMPLATE_BINDING_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn binding_ref(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        component_event: None,
        id: id.to_string(),
        event,
        mode: Default::default(),
        route: None,
        action: None,
        targets: Vec::new(),
    }
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
