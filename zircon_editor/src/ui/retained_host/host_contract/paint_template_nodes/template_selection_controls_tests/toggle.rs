use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::{
    control_border_color, selection_label_gap, toggle_thumb_color, toggle_thumb_rect,
    toggle_track_color, toggle_track_rect, TOGGLE_TRACK_WIDTH,
};
use super::support::{node_with_role, resolved_background_foreground_and_border};

#[test]
fn toggle_thumb_moves_to_checked_end_of_right_aligned_track() {
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 96.0,
        height: 28.0,
    };
    let node = TemplatePaneNodeData::default();
    let track = toggle_track_rect(&node, &rect);
    let unchecked = toggle_thumb_rect(&node, &track);
    let checked = toggle_thumb_rect(
        &TemplatePaneNodeData {
            checked: true,
            ..TemplatePaneNodeData::default()
        },
        &track,
    );

    assert_eq!(track.x, 58.0);
    assert_eq!(track.width, TOGGLE_TRACK_WIDTH);
    assert!(checked.x > unchecked.x);
    assert_eq!(unchecked.y, checked.y);
}

#[test]
fn toggle_honors_declared_track_and_thumb_metrics() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 96.0,
        height: 28.0,
    };
    let node = TemplatePaneNodeData {
        value_number: 40.0,
        layout_icon_size: 12.0,
        layout_content_offset_x: 8.0,
        layout_content_offset_y: 16.0,
        ..node_with_role("Toggle", "toggle", "WorkbenchToggleCustom")
    };
    let checked = TemplatePaneNodeData {
        checked: true,
        ..node.clone()
    };
    let track = toggle_track_rect(&node, &rect);
    let unchecked_thumb = toggle_thumb_rect(&node, &track);
    let checked_thumb = toggle_thumb_rect(&checked, &track);

    assert_eq!(track.x, 48.0);
    assert_eq!(track.y, 6.0);
    assert_eq!(track.width, 40.0);
    assert_eq!(track.height, 16.0);
    assert_eq!(unchecked_thumb.x, 50.0);
    assert_eq!(unchecked_thumb.y, 8.0);
    assert_eq!(unchecked_thumb.width, 12.0);
    assert_eq!(checked_thumb.x, 74.0);
    assert_eq!(selection_label_gap(&node), 8.0);
}

#[test]
fn toggle_consumes_declared_track_border_and_thumb_tones() {
    let checked = TemplatePaneNodeData {
        checked: true,
        selected: true,
        button_style: resolved_background_foreground_and_border(
            [53, 199, 208, 255],
            [255, 255, 255, 255],
            [49, 191, 201, 255],
        ),
        ..node_with_role("Toggle", "toggle", "WorkbenchToggleOn")
    };
    let unchecked = TemplatePaneNodeData {
        button_style: resolved_background_foreground_and_border(
            [15, 20, 23, 255],
            [124, 135, 142, 255],
            [53, 64, 71, 255],
        ),
        ..node_with_role("Toggle", "toggle", "WorkbenchToggleOff")
    };

    assert_eq!(toggle_track_color(&checked), [53, 199, 208, 255]);
    assert_eq!(toggle_thumb_color(&checked), [255, 255, 255, 255]);
    assert_eq!(control_border_color(&checked), [49, 191, 201, 255]);
    assert_eq!(toggle_track_color(&unchecked), [15, 20, 23, 255]);
    assert_eq!(toggle_thumb_color(&unchecked), [124, 135, 142, 255]);
    assert_eq!(control_border_color(&unchecked), [53, 64, 71, 255]);
}
