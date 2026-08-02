use crate::ui::layouts::common::model_rc;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::{
    TemplateNodePaintTransform, draw_template_nodes, draw_template_nodes_with_transform,
};
use super::support::{changed_pixel_count, panel_node, rect};

struct TestTransform;

struct ExactRowsTransform;

impl TemplateNodePaintTransform for TestTransform {
    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        mut clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        match node.control_id.as_str() {
            "moved" => node.frame.x += 12.0,
            "clipped" => clip.width = 4.0,
            "suppressed" => return None,
            _ => {}
        }
        Some((node, clip))
    }
}

impl TemplateNodePaintTransform for ExactRowsTransform {
    fn row_visit_indices(&self, _row_count: usize, _clip: &FrameRect) -> Option<Vec<usize>> {
        Some(vec![1])
    }

    fn transform(
        &self,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        Some((node, clip))
    }
}

#[test]
fn transform_defaults_to_visiting_the_complete_model() {
    let transform = TestTransform;

    assert_eq!(
        transform.row_visit_indices(4, &rect(0.0, 0.0, 10.0, 10.0)),
        None
    );
}

#[test]
fn transform_exact_row_plan_skips_unselected_model_rows() {
    let nodes = model_rc(vec![
        panel_node("skipped", 0.0, 0.0, 8.0, 8.0),
        panel_node("visited", 12.0, 0.0, 8.0, 8.0),
    ]);
    let bounds = rect(0.0, 0.0, 24.0, 12.0);
    let mut frame = HostRgbaFrame::filled(24, 12, [0, 0, 0, 255]);

    assert!(draw_template_nodes_with_transform(
        &mut frame,
        &nodes,
        &bounds,
        &bounds,
        None,
        Some(&ExactRowsTransform),
    ));

    assert_eq!(changed_pixel_count(frame.as_bytes(), 24, 0, 0, 8, 8), 0);
    assert!(changed_pixel_count(frame.as_bytes(), 24, 12, 0, 8, 8) > 0);
}

#[test]
fn template_node_paint_transform_moves_clips_and_suppresses_owned_nodes() {
    let nodes = model_rc(vec![
        panel_node("moved", 0.0, 0.0, 8.0, 8.0),
        panel_node("clipped", 0.0, 10.0, 12.0, 8.0),
        panel_node("suppressed", 20.0, 10.0, 8.0, 8.0),
    ]);
    let mut frame = HostRgbaFrame::filled(32, 20, [0, 0, 0, 255]);
    let bounds = rect(0.0, 0.0, 32.0, 20.0);

    let painted = draw_template_nodes_with_transform(
        &mut frame,
        &nodes,
        &bounds,
        &bounds,
        None,
        Some(&TestTransform),
    );

    assert!(painted);
    assert_eq!(changed_pixel_count(frame.as_bytes(), 32, 0, 0, 8, 8), 0);
    assert!(changed_pixel_count(frame.as_bytes(), 32, 12, 0, 8, 8) > 0);
    assert!(changed_pixel_count(frame.as_bytes(), 32, 0, 10, 4, 8) > 0);
    assert_eq!(changed_pixel_count(frame.as_bytes(), 32, 4, 10, 8, 8), 0);
    assert_eq!(changed_pixel_count(frame.as_bytes(), 32, 20, 10, 8, 8), 0);
    assert_eq!(nodes.row_data(0).expect("source moved node").frame.x, 0.0);
}

#[test]
fn template_node_paint_transform_none_matches_existing_draw_path() {
    let nodes = model_rc(vec![panel_node("identity", 4.0, 4.0, 12.0, 8.0)]);
    let bounds = rect(0.0, 0.0, 24.0, 16.0);
    let mut existing = HostRgbaFrame::filled(24, 16, [0, 0, 0, 255]);
    let mut transformed = HostRgbaFrame::filled(24, 16, [0, 0, 0, 255]);

    assert!(draw_template_nodes(
        &mut existing,
        &nodes,
        &bounds,
        &bounds,
        None,
    ));
    assert!(draw_template_nodes_with_transform(
        &mut transformed,
        &nodes,
        &bounds,
        &bounds,
        None,
        None,
    ));

    assert_eq!(existing.as_bytes(), transformed.as_bytes());
}

#[test]
fn template_node_transform_consumes_the_owned_model_row_without_a_second_clone() {
    let production = include_str!("../template_node_pipeline/draw.rs");

    assert!(!production.contains("source_node.clone()"));
}
