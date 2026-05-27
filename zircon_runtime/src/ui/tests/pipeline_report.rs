use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    pipeline::{UiPipelineDirtyReason, UiPipelineStage},
    tree::{UiInputPolicy, UiTreeNode},
};

#[test]
fn surface_frame_pipeline_report_uses_required_stage_order() {
    let mut surface = surface_with_button();

    surface.compute_layout(UiSize::new(120.0, 60.0)).unwrap();
    let frame = surface.surface_frame();
    let pipeline = &frame.pipeline_report;

    assert!(pipeline.is_complete_ordered());
    assert!(pipeline.missing_required_stages().is_empty());
    assert!(
        !pipeline
            .stage_report(UiPipelineStage::Layout)
            .unwrap()
            .skipped
    );
    assert!(
        !pipeline
            .stage_report(UiPipelineStage::PostLayout)
            .unwrap()
            .skipped
    );
    assert!(
        !pipeline
            .stage_report(UiPipelineStage::Picking)
            .unwrap()
            .skipped
    );
    assert!(
        !pipeline
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .skipped
    );
    assert!(
        pipeline
            .stage_report(UiPipelineStage::A11yExtract)
            .unwrap()
            .skipped
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::Layout)
            .unwrap()
            .counters
            .full_layout_count,
        1
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::PostLayout)
            .unwrap()
            .counters
            .stack_node_count,
        surface.arranged_tree.nodes.len() as u64
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::Picking)
            .unwrap()
            .counters
            .picking_candidate_count,
        surface.hit_test.grid.entries.len() as u64
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .counters
            .render_extract_command_count,
        surface.render_extract.list.commands.len() as u64
    );
}

#[test]
fn surface_frame_pipeline_report_keeps_render_only_rebuild_scoped() {
    let mut surface = surface_with_button();
    surface.rebuild();
    let previous_projection = surface.ui_ecs_projection();
    surface.clear_dirty_flags();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .dirty
        .render = true;
    let projection_mask = surface.ui_ecs_schedule_mask_from(&previous_projection);

    surface.rebuild_dirty(UiSize::new(120.0, 60.0)).unwrap();
    let pipeline = surface.surface_frame().pipeline_report;

    assert!(projection_mask.requires_stage(UiPipelineStage::RenderExtract));
    assert!(projection_mask.requires_stage(UiPipelineStage::BatchPrepare));
    assert!(!projection_mask.requires_stage(UiPipelineStage::Layout));
    assert!(!projection_mask.requires_stage(UiPipelineStage::PostLayout));
    assert!(!projection_mask.requires_stage(UiPipelineStage::Picking));
    assert!(pipeline.is_complete_ordered());
    assert!(
        pipeline
            .stage_report(UiPipelineStage::Layout)
            .unwrap()
            .skipped
    );
    assert!(
        pipeline
            .stage_report(UiPipelineStage::PostLayout)
            .unwrap()
            .skipped
    );
    assert!(
        pipeline
            .stage_report(UiPipelineStage::Picking)
            .unwrap()
            .skipped
    );
    assert!(
        !pipeline
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .skipped
    );
    assert_eq!(
        pipeline
            .stage_report(UiPipelineStage::RenderExtract)
            .unwrap()
            .dirty_reasons,
        vec![UiPipelineDirtyReason::Render]
    );
    assert_eq!(pipeline.totals.layout_node_count, 0);
    assert_eq!(
        pipeline.totals.render_extract_command_count,
        surface.render_extract.list.commands.len() as u64
    );
}

#[test]
fn debug_snapshot_carries_surface_pipeline_report() {
    let mut surface = surface_with_button();

    surface.compute_layout(UiSize::new(120.0, 60.0)).unwrap();
    let frame = surface.surface_frame();
    let snapshot = surface.debug_snapshot();

    assert_eq!(snapshot.pipeline_report, frame.pipeline_report);
    assert!(snapshot.pipeline_report.is_complete_ordered());
}

fn surface_with_button() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.pipeline"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
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
        .unwrap();
    surface
}
