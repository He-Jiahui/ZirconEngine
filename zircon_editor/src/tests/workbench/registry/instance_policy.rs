use std::hint::black_box;
use std::time::Instant;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewKind, ViewRegistry, WorkbenchSlot,
};

const DESCRIPTOR_BENCH_CAPABILITY_COUNT: usize = 256;
const DESCRIPTOR_BENCH_SAMPLE_COUNT: usize = 11;
const DESCRIPTOR_BENCH_ITERATIONS: usize = 2_000;

#[test]
fn view_registry_reuses_single_instance_and_allows_multi_instance() {
    let mut registry = ViewRegistry::default();
    registry
        .register_view(ViewDescriptor::new(
            ViewDescriptorId::new("editor.hierarchy"),
            ViewKind::ActivityView,
            "Hierarchy",
        ))
        .unwrap();
    registry
        .register_view(
            ViewDescriptor::new(
                ViewDescriptorId::new("editor.prefab"),
                ViewKind::ActivityWindow,
                "Prefab Editor",
            )
            .with_multi_instance(true)
            .with_workbench_slot(WorkbenchSlot::DocumentCenter),
        )
        .unwrap();

    let first = registry
        .open_descriptor(ViewDescriptorId::new("editor.hierarchy"))
        .unwrap();
    let second = registry
        .open_descriptor(ViewDescriptorId::new("editor.hierarchy"))
        .unwrap();
    let prefab_a = registry
        .open_descriptor(ViewDescriptorId::new("editor.prefab"))
        .unwrap();
    let prefab_b = registry
        .open_descriptor(ViewDescriptorId::new("editor.prefab"))
        .unwrap();

    assert_eq!(first.instance_id, second.instance_id);
    assert_ne!(prefab_a.instance_id, prefab_b.instance_id);
}

#[test]
fn workbench_slots_materialize_their_single_canonical_view_hosts() {
    let mut registry = ViewRegistry::default();
    let cases = [
        (WorkbenchSlot::LeftTopDrawer, "left-top"),
        (WorkbenchSlot::LeftBottomDrawer, "left-bottom"),
        (WorkbenchSlot::RightTopDrawer, "right-top"),
        (WorkbenchSlot::RightBottomDrawer, "right-bottom"),
        (WorkbenchSlot::BottomDrawer, "bottom"),
        (WorkbenchSlot::DocumentCenter, "document"),
        (WorkbenchSlot::FloatingWindow, "floating"),
        (WorkbenchSlot::ExclusiveMainPage, "exclusive"),
    ];
    for (slot, id) in cases {
        registry
            .register_view(
                ViewDescriptor::new(
                    ViewDescriptorId::new(format!("editor.slot.{id}")),
                    ViewKind::ActivityView,
                    id,
                )
                .with_workbench_slot(slot),
            )
            .unwrap();
        let instance = registry
            .open_descriptor(ViewDescriptorId::new(format!("editor.slot.{id}")))
            .unwrap();
        match (slot, instance.host) {
            (WorkbenchSlot::LeftTopDrawer, ViewHost::Drawer(ActivityDrawerSlot::LeftTop))
            | (WorkbenchSlot::LeftBottomDrawer, ViewHost::Drawer(ActivityDrawerSlot::LeftBottom))
            | (WorkbenchSlot::RightTopDrawer, ViewHost::Drawer(ActivityDrawerSlot::RightTop))
            | (
                WorkbenchSlot::RightBottomDrawer,
                ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
            )
            | (WorkbenchSlot::BottomDrawer, ViewHost::Drawer(ActivityDrawerSlot::Bottom))
            | (WorkbenchSlot::DocumentCenter, ViewHost::Document(_, _))
            | (WorkbenchSlot::FloatingWindow, ViewHost::FloatingWindow(_, _))
            | (WorkbenchSlot::ExclusiveMainPage, ViewHost::ExclusivePage(_)) => {}
            (slot, host) => panic!("slot {slot:?} materialized unexpected host {host:?}"),
        }
    }
}

#[test]
fn view_registry_open_and_restore_borrow_descriptor_metadata() {
    let open_source = include_str!("../../../ui/workbench/view/view_registry_open_descriptor.rs");
    let open_lookup = open_source
        .split("if let Some(error)")
        .next()
        .expect("open descriptor lookup section");
    let restore_source =
        include_str!("../../../ui/workbench/view/view_registry_restore_instance.rs");
    let restore_lookup = restore_source
        .split("if let Some(error)")
        .next()
        .expect("restore descriptor lookup section");
    assert!(!open_lookup.contains(".cloned()"));
    assert!(!restore_lookup.contains(".cloned()"));

    let descriptor_id = ViewDescriptorId::new("editor.borrowed_descriptor");
    let capability = "editor.borrowed_descriptor.enabled";
    let mut registry = ViewRegistry::default();
    registry.set_available_capabilities([capability]);
    registry
        .register_view(
            ViewDescriptor::new(
                descriptor_id.clone(),
                ViewKind::ActivityView,
                "Borrowed Descriptor",
            )
            .with_required_capabilities([capability]),
        )
        .unwrap();

    let first = registry.open_descriptor(descriptor_id.clone()).unwrap();
    let second = registry.open_descriptor(descriptor_id).unwrap();
    assert_eq!(first, second);
    let removed = registry.remove_instance(&first.instance_id).unwrap();
    assert_eq!(registry.restore_instance(removed).unwrap(), first);
}

#[test]
#[ignore = "managed release benchmark"]
fn view_registry_borrowed_descriptor_open_benchmark() {
    let (mut registry, descriptor_id) = descriptor_benchmark_registry();
    registry.open_descriptor(descriptor_id.clone()).unwrap();
    let mut retired_samples_ns = Vec::with_capacity(DESCRIPTOR_BENCH_SAMPLE_COUNT);
    let mut optimized_samples_ns = Vec::with_capacity(DESCRIPTOR_BENCH_SAMPLE_COUNT);

    for sample in 0..DESCRIPTOR_BENCH_SAMPLE_COUNT {
        if sample % 2 == 0 {
            retired_samples_ns.push(measure_retired_open(&mut registry, &descriptor_id));
            optimized_samples_ns.push(measure_borrowed_open(&mut registry, &descriptor_id));
        } else {
            optimized_samples_ns.push(measure_borrowed_open(&mut registry, &descriptor_id));
            retired_samples_ns.push(measure_retired_open(&mut registry, &descriptor_id));
        }
    }

    retired_samples_ns.sort_unstable();
    optimized_samples_ns.sort_unstable();
    let p95_index = (DESCRIPTOR_BENCH_SAMPLE_COUNT * 95).div_ceil(100) - 1;
    let retired_p95_ns = retired_samples_ns[p95_index];
    let optimized_p95_ns = optimized_samples_ns[p95_index];
    let reduction_percent = 100.0 * (1.0 - optimized_p95_ns as f64 / retired_p95_ns.max(1) as f64);
    eprintln!(
        "EDITOR52_VIEW_DESCRIPTOR_BORROW_BENCH_V1 samples={DESCRIPTOR_BENCH_SAMPLE_COUNT} iterations={DESCRIPTOR_BENCH_ITERATIONS} capability_strings={DESCRIPTOR_BENCH_CAPABILITY_COUNT} retired_descriptor_clones_per_open=1 optimized_descriptor_clones_per_open=0 retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} reduction_percent={reduction_percent:.3}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(75),
        "borrowed repeated-open P95 must be at most 75% of the retired descriptor clone path"
    );
}

fn descriptor_benchmark_registry() -> (ViewRegistry, ViewDescriptorId) {
    let descriptor_id = ViewDescriptorId::new("editor.descriptor_benchmark");
    let capabilities = (0..DESCRIPTOR_BENCH_CAPABILITY_COUNT)
        .map(|index| format!("editor.descriptor_benchmark.capability.{index:04}"))
        .collect::<Vec<_>>();
    let mut registry = ViewRegistry::default();
    registry.set_available_capabilities(capabilities.iter().cloned());
    registry
        .register_view(
            ViewDescriptor::new(
                descriptor_id.clone(),
                ViewKind::ActivityView,
                "Descriptor Benchmark",
            )
            .with_required_capabilities(capabilities),
        )
        .unwrap();
    (registry, descriptor_id)
}

fn measure_retired_open(registry: &mut ViewRegistry, descriptor_id: &ViewDescriptorId) -> u128 {
    let started = Instant::now();
    for _ in 0..DESCRIPTOR_BENCH_ITERATIONS {
        black_box(
            registry
                .open_descriptor_with_retired_clone_for_benchmark(descriptor_id.clone())
                .unwrap(),
        );
    }
    started.elapsed().as_nanos()
}

fn measure_borrowed_open(registry: &mut ViewRegistry, descriptor_id: &ViewDescriptorId) -> u128 {
    let started = Instant::now();
    for _ in 0..DESCRIPTOR_BENCH_ITERATIONS {
        black_box(registry.open_descriptor(descriptor_id.clone()).unwrap());
    }
    started.elapsed().as_nanos()
}
