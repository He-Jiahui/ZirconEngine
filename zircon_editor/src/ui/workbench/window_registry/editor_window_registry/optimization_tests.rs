use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use serde_json::Value;

use super::*;
use crate::ui::workbench::layout::{ActivityDrawerSlot, ActivityWindowHostMode};
use crate::ui::workbench::view::ViewHost;

const SAMPLE_PAIRS: usize = 21;

#[test]
fn editor13_window_registry_invalid_drawer_rebind_preserves_registry() {
    let (mut registry, drawer_id, _, _) = rebind_registry(64);
    let before = registry.clone();

    let error = registry
        .bind_drawer(DrawerBinding::new(
            ActivityWindowId::new("window:missing"),
            drawer_id,
            DrawerDockPosition::Bottom,
        ))
        .expect_err("missing target window must reject the rebind");

    assert!(error.contains("missing drawer owner window"));
    assert_eq!(registry, before, "failed rebind must be side-effect free");
}

#[test]
fn editor13_window_registry_borrowed_instance_index_preserves_lookup_identity() {
    let instances = view_instances(8, 64);
    let index = instances_by_id(&instances);

    for instance in &instances {
        let indexed = index
            .get(instance.instance_id.0.as_str())
            .expect("every source instance must be indexed");
        assert!(std::ptr::eq(*indexed, instance));
    }
}

#[test]
#[ignore = "release performance evidence"]
fn editor13_window_registry_borrowed_instance_index_benchmark_evidence() {
    const INSTANCES: usize = 16_384;
    const INSTANCE_ID_BYTES: usize = 256;

    let instances = view_instances(INSTANCES, INSTANCE_ID_BYTES);
    let mut legacy = || legacy_owned_instances_by_id(black_box(&instances)).len();
    let mut optimized = || instances_by_id(black_box(&instances)).len();

    assert_eq!(black_box(legacy()), INSTANCES);
    assert_eq!(black_box(optimized()), INSTANCES);
    let (legacy_ns, optimized_ns) = paired_samples(&mut legacy, &mut optimized, INSTANCES);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "borrowed instance index P95 must be at least 50% below the owned BTreeMap index: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR13_BORROWED_INSTANCE_INDEX_BENCH_V1 instances={INSTANCES} instance_id_bytes={INSTANCE_ID_BYTES} sample_pairs={SAMPLE_PAIRS} legacy_instance_id_clones={INSTANCES} optimized_instance_id_clones=0 legacy_cloned_instance_id_bytes={} optimized_cloned_instance_id_bytes=0 legacy_index=btree optimized_index=hash legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        INSTANCES * INSTANCE_ID_BYTES,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

#[test]
#[ignore = "release performance evidence"]
fn editor13_window_registry_clone_free_drawer_rebind_benchmark_evidence() {
    const TITLE_BYTES: usize = 65_536;
    const REBINDS: usize = 64;

    let (mut legacy_registry, drawer_id, first_window, second_window) =
        rebind_registry(TITLE_BYTES);
    let mut optimized_registry = legacy_registry.clone();
    let mut legacy = || {
        alternating_rebinds(
            &mut legacy_registry,
            &drawer_id,
            &first_window,
            &second_window,
            REBINDS,
            legacy_bind_drawer,
        )
    };
    let mut optimized = || {
        alternating_rebinds(
            &mut optimized_registry,
            &drawer_id,
            &first_window,
            &second_window,
            REBINDS,
            EditorWindowRegistry::bind_drawer,
        )
    };

    assert_eq!(black_box(legacy()), TITLE_BYTES);
    assert_eq!(black_box(optimized()), TITLE_BYTES);
    let (legacy_ns, optimized_ns) = paired_samples(&mut legacy, &mut optimized, TITLE_BYTES);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(3),
        "clone-free drawer rebind P95 must be at least 40% below whole-instance cloning: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR13_CLONE_FREE_DRAWER_REBIND_BENCH_V1 title_bytes={TITLE_BYTES} rebinds={REBINDS} sample_pairs={SAMPLE_PAIRS} legacy_drawer_instance_clones={REBINDS} optimized_drawer_instance_clones=0 legacy_cloned_title_bytes={} optimized_cloned_title_bytes=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        TITLE_BYTES * REBINDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn view_instances(count: usize, instance_id_bytes: usize) -> Vec<ViewInstance> {
    let suffix_bytes = instance_id_bytes.saturating_sub(6);
    (0..count)
        .map(|index| ViewInstance {
            instance_id: ViewInstanceId::new(format!("{index:05}-{}", "x".repeat(suffix_bytes))),
            descriptor_id: ViewDescriptorId::new("editor.fixture"),
            title: "Fixture".to_string(),
            serializable_payload: Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
        })
        .collect()
}

fn legacy_owned_instances_by_id(
    instances: &[ViewInstance],
) -> BTreeMap<ViewInstanceId, &ViewInstance> {
    instances
        .iter()
        .map(|instance| (instance.instance_id.clone(), instance))
        .collect()
}

fn rebind_registry(
    title_bytes: usize,
) -> (
    EditorWindowRegistry,
    ViewInstanceId,
    ActivityWindowId,
    ActivityWindowId,
) {
    let first_window = ActivityWindowId::new("window:first");
    let second_window = ActivityWindowId::new("window:second");
    let drawer_id = ViewInstanceId::new("editor.inspector#1");
    let mut registry = EditorWindowRegistry::default();
    for window_id in [&first_window, &second_window] {
        registry.register_window(WindowInstance::new(
            window_id.clone(),
            ViewDescriptorId::new("editor.fixture_window"),
            WindowKind::DrawerCapable,
            "Fixture",
            ActivityWindowHostMode::EmbeddedMainFrame,
        ));
    }
    registry
        .register_drawer_view(DrawerViewInstance::new(
            drawer_id.clone(),
            ViewDescriptorId::new("editor.inspector"),
            "x".repeat(title_bytes),
            first_window.clone(),
            DrawerDockPosition::RightTop,
        ))
        .expect("fixture drawer must register");
    (registry, drawer_id, first_window, second_window)
}

fn alternating_rebinds(
    registry: &mut EditorWindowRegistry,
    drawer_id: &ViewInstanceId,
    first_window: &ActivityWindowId,
    second_window: &ActivityWindowId,
    rebinds: usize,
    mut bind: impl FnMut(&mut EditorWindowRegistry, DrawerBinding) -> Result<(), String>,
) -> usize {
    for index in 0..rebinds {
        let target = if index % 2 == 0 {
            second_window
        } else {
            first_window
        };
        bind(
            registry,
            DrawerBinding::new(
                target.clone(),
                drawer_id.clone(),
                DrawerDockPosition::Bottom,
            ),
        )
        .expect("valid alternating drawer rebind");
    }
    registry
        .get_drawer_view(drawer_id)
        .expect("rebound drawer")
        .title
        .len()
}

fn legacy_bind_drawer(
    registry: &mut EditorWindowRegistry,
    binding: DrawerBinding,
) -> Result<(), String> {
    let drawer = registry
        .drawer_views
        .get_mut(&binding.drawer_view)
        .ok_or_else(|| format!("missing drawer view {}", binding.drawer_view.0))?;
    let old_owner = drawer.owner_window.clone();
    if let Some(old_window) = registry.windows.get_mut(&old_owner) {
        for views in old_window.drawer_views.values_mut() {
            views.retain(|view| view != &binding.drawer_view);
        }
        if old_window.selected_drawer.as_ref() == Some(&binding.drawer_view) {
            old_window.selected_drawer = None;
        }
    }
    drawer.owner_window = binding.window_id.clone();
    drawer.dock_position = binding.dock_position;
    let rebound = drawer.clone();
    registry.register_drawer_view(rebound)
}

fn paired_samples(
    legacy: &mut impl FnMut() -> usize,
    optimized: &mut impl FnMut() -> usize,
    expected: usize,
) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(legacy, expected));
            optimized_ns.push(measure_ns(optimized, expected));
        } else {
            optimized_ns.push(measure_ns(optimized, expected));
            legacy_ns.push(measure_ns(legacy, expected));
        }
    }
    (legacy_ns, optimized_ns)
}

fn measure_ns(operation: &mut impl FnMut() -> usize, expected: usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(operation()), expected);
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
