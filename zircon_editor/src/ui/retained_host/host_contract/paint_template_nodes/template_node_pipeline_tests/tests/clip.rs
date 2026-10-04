use crate::ui::layouts::common::model_rc;

use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::draw_template_nodes;
use super::support::{changed_pixel_count, panel_node, rect};

#[test]
fn template_nodes_skip_when_active_paint_clip_misses_template_clip() {
    let mut frame = HostRgbaFrame::filled(32, 32, [1, 2, 3, 255]);
    let before = frame.as_bytes().to_vec();
    frame.replace_paint_clip(Some(rect(24.0, 24.0, 4.0, 4.0)));

    let bounds = rect(0.0, 0.0, 16.0, 16.0);
    let painted = draw_template_nodes(
        &mut frame,
        &model_rc(vec![panel_node("outside", 0.0, 0.0, 8.0, 8.0)]),
        &bounds,
        &bounds,
        None,
    );

    assert!(!painted);
    assert_eq!(frame.as_bytes(), before.as_slice());
}

#[test]
fn template_nodes_only_paint_nodes_intersecting_active_damage_clip() {
    let mut frame = HostRgbaFrame::filled(40, 20, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(rect(20.0, 0.0, 10.0, 10.0)));

    let bounds = rect(0.0, 0.0, 40.0, 20.0);
    let painted = draw_template_nodes(
        &mut frame,
        &model_rc(vec![
            panel_node("left", 0.0, 0.0, 10.0, 10.0),
            panel_node("damage", 20.0, 0.0, 10.0, 10.0),
        ]),
        &bounds,
        &bounds,
        None,
    );

    assert!(painted);
    assert_eq!(changed_pixel_count(frame.as_bytes(), 40, 0, 0, 10, 10), 0);
    assert!(changed_pixel_count(frame.as_bytes(), 40, 20, 0, 10, 10) > 0);
}
