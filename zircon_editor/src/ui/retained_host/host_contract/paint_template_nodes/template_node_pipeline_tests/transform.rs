use crate::ui::layouts::common::model_rc;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::{
    draw_template_nodes, draw_template_nodes_with_transform, TemplateNodePaintTransform,
};
use super::support::{changed_pixel_count, panel_node, rect};

struct TestTransform;

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
