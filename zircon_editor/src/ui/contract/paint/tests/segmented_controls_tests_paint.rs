use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_segmented_control_geometry::{
    segment_rect, segmented_body_rect, selected_segment_rect, selected_segment_underline_rect,
};
use super::super::push_segmented_control_commands;
use super::super::style::{
    segmented_background, selected_segment_underline_height, SEGMENT_IDLE_BACKGROUND,
    SEGMENT_SELECTED_BACKGROUND,
};
use super::support::{
    changed_pixel_count, frame_rect, labeled_segmented_node, pixel_at, segmented_node,
};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn segmented_control_paints_selected_middle_segment() {
    let node = segmented_node();
    let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![node.clone()]));
    let body = segmented_body_rect(&node, &frame_rect(&node.frame));
    let selected_segment = segment_rect(&body, 1, node.options.row_count());
    let selected = selected_segment_rect(&selected_segment);
    let underline =
        selected_segment_underline_rect(&selected, selected_segment_underline_height(&node));
    let selected_fill_x = (selected.x + selected.width * 0.12).round() as u32;
    let selected_fill_y = (selected.y + 3.0).round() as u32;

    assert_eq!(
        segmented_background(&segmented_node()),
        SEGMENT_IDLE_BACKGROUND
    );
    assert_eq!(pixel_at(&bytes, 180, 17, 15), SEGMENT_IDLE_BACKGROUND);
    assert_eq!(
        pixel_at(&bytes, 180, selected_fill_x, selected_fill_y),
        SEGMENT_SELECTED_BACKGROUND
    );
    assert_ne!(
        pixel_at(&bytes, 180, selected_fill_x, selected_fill_y),
        PALETTE.surface_selected
    );
    assert_eq!(
        pixel_at(
            &bytes,
            180,
            (underline.x + underline.width * 0.5).round() as u32,
            (underline.y + underline.height * 0.5).round() as u32,
        ),
        PALETTE.accent
    );
    assert!(changed_pixel_count(&bytes, 180, 62, 8, 48, 22) > 0);
    assert!(changed_pixel_count(&bytes, 180, 14, 8, 40, 22) > 0);
}

#[test]
fn segmented_control_paints_group_label_and_body() {
    let bytes = paint_template_nodes_for_test(190, 60, model_rc(vec![labeled_segmented_node()]));

    assert!(changed_pixel_count(&bytes, 190, 12, 4, 132, 14) > 0);
    assert!(changed_pixel_count(&bytes, 190, 18, 22, 144, 30) > 0);
    assert_eq!(pixel_at(&bytes, 190, 12, 22), [0, 0, 0, 255]);
}

#[test]
fn fully_clipped_segmented_control_does_not_emit_paint_commands() {
    let node = segmented_node();
    let rect = frame_rect(&node.frame);
    let clip = FrameRect {
        x: 180.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_segmented_control_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_segmented_control_keeps_clipped_label_commands() {
    let node = segmented_node();
    let rect = frame_rect(&node.frame);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_segmented_control_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(commands.iter().any(|command| command.text.is_some()));
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&clip)));
}
