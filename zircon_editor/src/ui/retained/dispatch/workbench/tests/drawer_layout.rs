use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::workbench::fixture::default_preview_fixture;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::UiTreeNode,
};

#[test]
fn collapsed_bottom_drawer_input_uses_the_callers_panel_header_metric() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let mut model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let bottom = model
        .drawer_ring
        .drawers
        .get_mut(&ActivityDrawerSlot::Bottom)
        .expect("preview fixture should expose a bottom drawer");
    bottom.mode = ActivityDrawerMode::Collapsed;
    bottom.visible = true;
    assert!(!bottom.tabs.is_empty());

    let metrics = WorkbenchChromeMetrics {
        panel_header_height: 31.0,
        ..WorkbenchChromeMetrics::default()
    };
    let inputs = WorkbenchDrawerLayoutInputs::from_workbench_model(&model, &metrics);

    assert!(inputs.bottom.visible);
    assert_eq!(inputs.bottom.extent, metrics.panel_header_height);
}

#[test]
fn narrow_width_collapses_a_visible_bottom_drawer_to_its_tab_strip() {
    let pinned_drawer = WorkbenchDrawerRegionInput {
        visible: true,
        extent: 228.0,
    };
    let metrics = WorkbenchChromeMetrics {
        panel_header_height: 31.0,
        ..WorkbenchChromeMetrics::default()
    };
    let compacted = compacted_bottom_region_input(
        pinned_drawer,
        UiSize::new(640.0, 520.0),
        metrics,
        Some(WorkbenchDrawerLayoutAnchors { body_height: 420.0 }),
    );

    assert_eq!(compacted.extent, metrics.panel_header_height);
}

#[test]
fn stable_drawer_projection_does_not_dirty_template_roots() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = UiSize::new(900.0, 620.0);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size).unwrap();
    let inputs = WorkbenchDrawerLayoutInputs::from_workbench_model(&model, &metrics);
    apply_workbench_drawer_layout(
        &mut bridge.template_surface.surface,
        shell_size,
        inputs,
        metrics,
        None,
        false,
    )
    .unwrap();
    bridge.template_surface.surface.clear_dirty_flags();

    apply_workbench_drawer_layout(
        &mut bridge.template_surface.surface,
        shell_size,
        inputs,
        metrics,
        None,
        false,
    )
    .unwrap();

    assert!(bridge
        .template_surface
        .surface
        .tree
        .pending_layout_source_node_ids()
        .is_empty());
}

#[test]
fn mount_changes_dirty_every_root_while_stable_projection_cost_is_constant() {
    for root_count in [1_usize, 64, 1024] {
        let mut surface = surface_with_roots(root_count);
        mark_roots_layout_dirty_if_needed(&mut surface, false).unwrap();
        assert_eq!(surface.tree.pending_layout_source_node_ids().len(), 0);
        mark_roots_layout_dirty_if_needed(&mut surface, true).unwrap();
        assert_eq!(
            surface.tree.pending_layout_source_node_ids().len(),
            root_count
        );
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260905_editor_stable_root_invalidation_bench() {
    const SAMPLE_PAIRS: usize = 21;
    const INVALIDATIONS_PER_SAMPLE: usize = 256;
    let base = surface_with_roots(1024);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_root_invalidation(
                base.clone(),
                true,
                INVALIDATIONS_PER_SAMPLE,
            ));
            optimized_samples.push(measure_root_invalidation(
                base.clone(),
                false,
                INVALIDATIONS_PER_SAMPLE,
            ));
        } else {
            optimized_samples.push(measure_root_invalidation(
                base.clone(),
                false,
                INVALIDATIONS_PER_SAMPLE,
            ));
            legacy_samples.push(measure_root_invalidation(
                base.clone(),
                true,
                INVALIDATIONS_PER_SAMPLE,
            ));
        }
    }
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR_STABLE_ROOT_INVALIDATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
roots=1024 invalidations_per_sample={INVALIDATIONS_PER_SAMPLE} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}"
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn surface_with_roots(root_count: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("editor.drawer-layout.root-bench"));
    for index in 0..root_count {
        let node_id = UiNodeId::new(index as u64 + 1);
        surface.tree.insert_root(UiTreeNode::new(
            node_id,
            UiNodePath::new(format!("root/{index}")),
        ));
    }
    surface.clear_dirty_flags();
    surface
}

fn measure_root_invalidation(
    mut surface: UiSurface,
    invalidate_roots: bool,
    iterations: usize,
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        mark_roots_layout_dirty_if_needed(black_box(&mut surface), invalidate_roots).unwrap();
    }
    black_box(surface.tree.pending_layout_source_node_ids().len());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
