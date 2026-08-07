#![cfg(feature = "ui")]

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, LayoutBoundary, StretchMode, UiContainerKind, UiFrame,
        UiSize,
    },
    pipeline::UiPipelineStage,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn single_node_layout_dirty_reports_full_post_layout_outer_visits() {
    let mut surface = flat_surface(2);
    mark_last_child_layout_dirty(&mut surface, 2);

    let report = surface.rebuild_dirty(root_size()).unwrap();
    let pipeline = report.pipeline_report(1);

    assert_eq!(report.layout_visited_node_count, 1);
    assert_eq!(report.arranged_outer_node_visit_count, 3);
    assert_eq!(report.hit_grid_outer_node_visit_count, 3);
    assert_eq!(report.render_outer_node_visit_count, 3);
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::PostLayout)
            .unwrap()
            .counters
            .post_layout_outer_node_visit_count,
        3
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::Picking)
            .unwrap()
            .counters
            .picking_outer_node_visit_count,
        3
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .counters
            .render_extract_outer_node_visit_count,
        3
    );
}

#[test]
fn surface_report_exposes_text_cache_hits_and_misses() {
    let mut surface = text_surface();
    surface
        .tree
        .node_mut(child_id(0))
        .expect("text node should exist")
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(
            r#"
text = "Measured label"
editable_text = true
font_size = 10.0
line_height = 12.0
wrap = "Word"
"#,
        )
        .expect("text metadata should parse"),
        ..UiTemplateNodeMetadata::default()
    });

    surface.compute_layout(root_size()).unwrap();
    let first = surface.surface_frame().pipeline_report;
    let first_text = first
        .stage_report(UiPipelineStage::TextMeasure)
        .expect("text stage should be reported");

    assert!(!first_text.skipped);
    assert_eq!(first_text.counters.text_measure_cache_miss_count, 1);
    assert_eq!(
        first_text.counters.text_layout_cache_miss_count, 1,
        "first text counters={:?}, render_commands={:?}",
        first_text.counters, surface.render_extract.list.commands
    );
    assert!(first_text.counters.text_shape_cache_miss_count > 0);

    surface.rebuild();
    let forced_rebuild = surface.surface_frame().pipeline_report;
    let forced_rebuild_text = forced_rebuild
        .stage_report(UiPipelineStage::TextMeasure)
        .expect("forced-rebuild text stage should be reported");

    assert!(!forced_rebuild_text.skipped);
    assert_eq!(forced_rebuild_text.counters.text_layout_cache_hit_count, 1);
    assert_eq!(forced_rebuild_text.counters.text_layout_cache_miss_count, 0);
    assert_eq!(forced_rebuild_text.counters.text_shape_cache_miss_count, 0);
}

fn text_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.incremental_refresh.text"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                ..UiStateFlags::default()
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(child_id(0), UiNodePath::new("root/text"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
        )
        .expect("text node should insert");
    surface
}

#[test]
#[ignore = "explicit M0 scale matrix; run at milestone performance gates"]
fn stable_and_single_node_dirty_scale_matrix_exposes_post_layout_outer_traversals() {
    for child_count in [1_usize, 100, 10_000] {
        let mut surface = flat_surface(child_count);
        let stable = surface.rebuild_dirty(root_size()).unwrap();
        assert!(!stable.dirty_flags.any(), "{child_count}");
        assert!(!stable.layout_recomputed, "{child_count}");
        assert!(!stable.arranged_rebuilt, "{child_count}");
        assert!(!stable.hit_grid_rebuilt, "{child_count}");
        assert!(!stable.render_rebuilt, "{child_count}");
        assert_eq!(stable.layout_visited_node_count, 0, "{child_count}");
        assert_eq!(stable.arranged_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.hit_grid_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.render_outer_node_visit_count, 0, "{child_count}");
        assert_eq!(stable.text_measure_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_measure_cache_miss_count, 0, "{child_count}");
        assert_eq!(stable.text_layout_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_layout_cache_miss_count, 0, "{child_count}");
        assert_eq!(stable.text_shape_cache_hit_count, 0, "{child_count}");
        assert_eq!(stable.text_shape_cache_miss_count, 0, "{child_count}");

        mark_last_child_layout_dirty(&mut surface, child_count);

        let report = surface.rebuild_dirty(root_size()).unwrap();
        let total_node_count = child_count + 1;

        assert_eq!(report.layout_visited_node_count, 1, "{child_count}");
        assert_eq!(
            report.arranged_outer_node_visit_count, total_node_count,
            "{child_count}"
        );
        assert_eq!(
            report.hit_grid_outer_node_visit_count, total_node_count,
            "{child_count}"
        );
        assert_eq!(
            report.render_outer_node_visit_count, total_node_count,
            "{child_count}"
        );
        eprintln!(
            "nodes={total_node_count} layout_visited={} arranged_outer_visited={} hit_outer_visited={} render_outer_visited={} layout_us={} arranged_us={} hit_us={} render_us={}",
            report.layout_visited_node_count,
            report.arranged_outer_node_visit_count,
            report.hit_grid_outer_node_visit_count,
            report.render_outer_node_visit_count,
            report.layout_elapsed_micros,
            report.arranged_elapsed_micros,
            report.hit_grid_elapsed_micros,
            report.render_elapsed_micros,
        );
    }
}

fn flat_surface(child_count: usize) -> UiSurface {
    let mut surface = unbuilt_flat_surface(child_count);
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn unbuilt_flat_surface(child_count: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.incremental_refresh.scale.{child_count}"
    )));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_constraints(BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            })
            .with_container(UiContainerKind::Free)
            .with_layout_boundary(LayoutBoundary::ParentDirected),
    );
    for child_index in 0..child_count {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    child_id(child_index),
                    UiNodePath::new(format!("root/{child_index}")),
                )
                .with_constraints(BoxConstraints {
                    width: fixed_constraint(40.0),
                    height: fixed_constraint(20.0),
                })
                .with_layout_boundary(LayoutBoundary::ParentDirected),
            )
            .expect("scale-matrix node should insert");
    }
    surface
}

fn mark_last_child_layout_dirty(surface: &mut UiSurface, child_count: usize) {
    let changed_node_id = child_id(child_count - 1);
    surface
        .tree
        .node_mut(changed_node_id)
        .expect("changed node should exist")
        .constraints
        .width = fixed_constraint(60.0);
    surface
        .tree
        .node_mut(changed_node_id)
        .expect("changed node should exist")
        .dirty
        .layout = true;
}

fn child_id(child_index: usize) -> UiNodeId {
    UiNodeId::new(child_index as u64 + 2)
}

fn fixed_constraint(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: value,
        preferred: value,
        max: value,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn root_size() -> UiSize {
    UiSize::new(120.0, 60.0)
}
