use super::*;
use std::sync::Arc;

#[cfg(feature = "profiling")]
use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};
use zircon_runtime_interface::ui::{
    pipeline::UiPipelineStage, surface::UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE,
};

#[test]
fn render_patch_publishes_changed_commands_and_retains_other_segments() {
    let (mut surface, size) = segmented_button_surface();
    let before = surface.surface_frame();
    let stable_id = UiNodeId::new(101);
    let changed_id = UiNodeId::new(225);
    let stable_index = before
        .render_extract
        .list
        .commands
        .iter()
        .position(|command| command.node_id == stable_id)
        .unwrap();
    let changed_index = before
        .render_extract
        .list
        .commands
        .iter()
        .position(|command| command.node_id == changed_id)
        .unwrap();
    assert_ne!(
        stable_index / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE,
        changed_index / UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE,
    );
    assert_eq!(
        before.render_extract.list.commands[changed_index].opacity,
        1.0
    );

    {
        let mut node = surface.tree.node_mut(changed_id).unwrap();
        node.template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert("opacity".to_string(), toml::Value::Float(0.375));
        node.dirty.render = true;
    }
    let report = surface.rebuild_dirty(size).unwrap();
    let after = surface.surface_frame();

    assert!(report.render_patched);
    assert!(report.render_rebuilt);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert_eq!(
        after.render_extract.list.commands.len(),
        before.render_extract.list.commands.len()
    );
    assert_eq!(
        after.render_extract.list.commands[changed_index].opacity,
        0.375
    );
    assert_eq!(
        before.render_extract.list.commands[changed_index].opacity,
        1.0
    );
    assert_eq!(after.render_extract.to_extract(), surface.render_extract);
    assert!(!Arc::ptr_eq(&before.render_extract, &after.render_extract));
    assert!(std::ptr::eq(
        &before.render_extract.list.commands[stable_index],
        &after.render_extract.list.commands[stable_index],
    ));
    assert!(!std::ptr::eq(
        &before.render_extract.list.commands[changed_index],
        &after.render_extract.list.commands[changed_index],
    ));
    assert!(after.domain_generations.render > before.domain_generations.render);
    assert_eq!(
        after.domain_generations.layout,
        before.domain_generations.layout
    );
    assert_eq!(
        after.domain_generations.hit_test,
        before.domain_generations.hit_test
    );
    assert!(Arc::ptr_eq(&before.arranged_tree, &after.arranged_tree));
    assert!(Arc::ptr_eq(&before.hit_grid, &after.hit_grid));
    assert!(Arc::ptr_eq(&after, &surface.surface_frame()));

    let stage = after
        .pipeline_report
        .stage_report(UiPipelineStage::RenderExtract)
        .unwrap();
    assert!(!stage.skipped);
    assert_eq!(stage.elapsed_micros, report.render_elapsed_micros);
    assert_eq!(stage.counters.render_extract_outer_node_visit_count, 1);
    assert_eq!(
        stage.counters.render_command_rebuild_count,
        report.render_command_rebuilt_count as u64
    );
}

#[cfg(feature = "profiling")]
#[test]
#[ignore = "global capture requires an isolated serial run"]
fn render_patch_records_the_owned_elapsed_counter() {
    let _capture_lock = test_capture_lock();
    let (mut surface, size) = segmented_button_surface();
    {
        let mut node = surface.tree.node_mut(UiNodeId::new(225)).unwrap();
        node.template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert("opacity".to_string(), toml::Value::Float(0.375));
        node.dirty.render = true;
    }
    reset_capture();
    start_capture(ProfileCaptureConfig::default());
    let report = surface.rebuild_dirty(size).unwrap();
    let profile = snapshot();
    reset_capture();

    assert!(report.render_patched);
    let samples = profile
        .counters
        .iter()
        .filter(|counter| counter.name == "ui.surface_rebuild.render_extract_elapsed_us")
        .collect::<Vec<_>>();
    assert_eq!(
        samples.len(),
        1,
        "the isolated patch must record exactly its own elapsed time"
    );
    assert_eq!(samples[0].value, report.render_elapsed_micros as f64);
}

#[test]
fn interaction_patch_publishes_updated_arranged_state() {
    let mut surface = sibling_surface(UiContainerKind::Free, LayoutBoundary::ParentDirected);
    let before = surface.surface_frame();
    assert!(before.arranged_tree.get(primary_id()).unwrap().enabled);
    surface
        .tree
        .node_mut(primary_id())
        .unwrap()
        .state_flags
        .enabled = false;
    surface
        .invalidate_node(primary_id(), UiInvalidationReason::Interaction)
        .unwrap();

    let report = surface.rebuild_dirty(root_size()).unwrap();
    let after = surface.surface_frame();
    assert!(report.arranged_rebuilt);
    assert!(report.arranged_patched);
    assert!(report.hit_grid_patched);
    assert!(!report.layout_recomputed);
    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert!(!after.arranged_tree.get(primary_id()).unwrap().enabled);
    assert!(before.arranged_tree.get(primary_id()).unwrap().enabled);
    assert!(!Arc::ptr_eq(&before.arranged_tree, &after.arranged_tree));
    assert!(after.domain_generations.layout > before.domain_generations.layout);
    let stage = after
        .pipeline_report
        .stage_report(UiPipelineStage::PostLayout)
        .unwrap();
    assert!(!stage.skipped);
    assert_eq!(stage.counters.post_layout_outer_node_visit_count, 1);
    let picking = after
        .pipeline_report
        .stage_report(UiPipelineStage::Picking)
        .unwrap();
    assert!(!picking.skipped);
    assert_eq!(picking.counters.hit_grid_rebuild_count, 0);
}

#[test]
fn successful_patch_stages_report_their_work_even_without_full_rebuilds() {
    let report = UiSurfaceRebuildReport {
        arranged_patched: true,
        hit_grid_patched: true,
        render_patched: true,
        arranged_outer_node_visit_count: 2,
        hit_grid_outer_node_visit_count: 0,
        render_outer_node_visit_count: 1,
        render_command_rebuilt_count: 3,
        arranged_elapsed_micros: 31,
        hit_grid_elapsed_micros: 37,
        render_elapsed_micros: 41,
        ..Default::default()
    };
    let pipeline = report.pipeline_report(7);
    for (stage, elapsed) in [
        (UiPipelineStage::PostLayout, 31),
        (UiPipelineStage::Picking, 37),
        (UiPipelineStage::RenderExtract, 41),
    ] {
        let stage = pipeline.stage_report(stage).unwrap();
        assert!(!stage.skipped);
        assert_eq!(stage.elapsed_micros, elapsed);
    }
    let picking = pipeline.stage_report(UiPipelineStage::Picking).unwrap();
    assert_eq!(picking.counters.hit_grid_rebuild_count, 0);
    assert_eq!(picking.counters.picking_outer_node_visit_count, 0);
    assert_eq!(pipeline.totals.post_layout_outer_node_visit_count, 2);
    assert_eq!(pipeline.totals.render_command_rebuild_count, 3);

    let idle = UiSurfaceRebuildReport::default().pipeline_report(8);
    for stage in [
        UiPipelineStage::PostLayout,
        UiPipelineStage::Picking,
        UiPipelineStage::RenderExtract,
    ] {
        assert!(idle.stage_report(stage).unwrap().skipped);
    }
}

fn segmented_button_surface() -> (UiSurface, UiSize) {
    let size = UiSize::new(512.0, 512.0);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.patch_publication"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, size.width, size.height))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                ..Default::default()
            }),
    );
    for offset in 0..130 {
        surface
            .tree
            .insert_child(
                root_id(),
                UiTreeNode::new(
                    UiNodeId::new(100 + offset),
                    UiNodePath::new(format!("root/item_{offset}")),
                )
                .with_frame(UiFrame::new(0.0, offset as f32 * 2.0, 32.0, 2.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MaterialButton".to_string(),
                    attributes: toml::from_str(
                        "text = 'Run'\nopacity = 1.0\n[background]\ncolor = '#2f6f5e'",
                    )
                    .unwrap(),
                    ..Default::default()
                }),
            )
            .unwrap();
    }
    surface.rebuild_authored_frames(size);
    surface.clear_dirty_flags();
    (surface, size)
}
