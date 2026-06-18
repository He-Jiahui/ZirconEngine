use super::super::super::data::TemplateNodeFrameData;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::{Color, SharedString};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn segmented_options_prefer_declared_option_cells() {
    let node = segmented_node();

    assert_eq!(
        segmented_options(&node),
        vec![
            "left".to_string(),
            "center".to_string(),
            "right".to_string()
        ]
    );
    assert_eq!(selected_segment_value(&node).as_deref(), Some("center"));
}

#[test]
fn segment_rects_split_available_width_evenly() {
    let rect = FrameRect {
        x: 6.0,
        y: 4.0,
        width: 150.0,
        height: 30.0,
    };

    assert_eq!(segment_rect(&rect, 0, 3).x, 6.0);
    assert_eq!(segment_rect(&rect, 1, 3).x, 56.0);
    assert_eq!(segment_rect(&rect, 2, 3).width, 50.0);
}

#[test]
fn selected_segment_style_defaults_to_legacy_border_without_declaration() {
    let node = segmented_node();

    assert_eq!(selected_segment_border_width(&node), 1.0);
    assert_eq!(selected_segment_underline_height(&node), 0.0);
    assert_eq!(selected_segment_underline_color(&node), PALETTE.accent);
}

#[test]
fn selected_segment_style_honors_declared_border_suppression_and_underline() {
    let mut node = segmented_node();
    node.has_selected_segment_border_width = true;
    node.selected_segment_border_width = 0.0;
    node.selected_segment_underline_height = 1.0;
    node.selected_segment_underline_color = Color::from_argb_u8(122, 50, 211, 222);

    assert_eq!(selected_segment_border_width(&node), 0.0);
    assert_eq!(selected_segment_underline_height(&node), 1.0);
    assert_eq!(selected_segment_underline_color(&node), [50, 211, 222, 122]);
}

#[test]
fn segmented_control_paints_selected_middle_segment() {
    let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![segmented_node()]));

    assert_eq!(
        segmented_background(&segmented_node()),
        SEGMENT_IDLE_BACKGROUND
    );
    assert_eq!(pixel_at(&bytes, 180, 17, 15), SEGMENT_IDLE_BACKGROUND);
    assert!(changed_pixel_count(&bytes, 180, 62, 8, 48, 22) > 0);
    assert!(changed_pixel_count(&bytes, 180, 14, 8, 40, 22) > 0);
}

#[test]
fn segmented_control_paints_group_label_and_offsets_body() {
    let node = labeled_segmented_node();
    let body = segmented_body_rect(&node, &frame_rect(&node.frame));

    assert_eq!(body.x, 18.0);
    assert_eq!(body.y, 22.0);
    assert_eq!(body.height, 30.0);

    let bytes = paint_template_nodes_for_test(190, 60, model_rc(vec![node]));

    assert!(changed_pixel_count(&bytes, 190, 12, 4, 132, 14) > 0);
    assert!(changed_pixel_count(&bytes, 190, 18, 22, 144, 30) > 0);
    assert_eq!(pixel_at(&bytes, 190, 12, 22), [0, 0, 0, 255]);
}

#[test]
fn selected_tab_paints_accent_underline_without_filling_right_edge() {
    let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![tab_node()]));

    assert!(changed_pixel_count(&bytes, 180, 0, 40, 150, 4) > 0);
    assert_eq!(pixel_at(&bytes, 180, 148, 8), [0, 0, 0, 255]);
}

#[test]
fn selected_tab_honors_declared_layout_offset() {
    let mut node = tab_node();
    node.control_id = "WorkbenchLabsTabOne".into();
    node.layout_offset_x = 3.0;
    node.layout_offset_y = 2.0;
    let paint_rect = tab_paint_rect(&node, &frame_rect(&node.frame));

    assert!(is_workbench_tab(&node));
    assert_eq!(paint_rect.x, 3.0);
    assert_eq!(paint_rect.y, 6.0);

    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 180, 0, 44), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 180, 3, 44, 150, 2) > 0);
}

#[test]
fn workbench_tab_uses_declared_idle_background() {
    use zircon_runtime_interface::ui::style::{
        ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
    };

    let mut node = tab_node();
    node.control_id = "WorkbenchLabsTabs".into();
    node.text = "".into();
    node.checked = false;
    node.selected = false;
    node.button_style = ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 34, 38, 255))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    };

    assert!(is_workbench_tab(&node));
    assert_eq!(tab_background(&node), Some([28, 34, 38, 255]));

    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 180, 8, 12), [28, 34, 38, 255]);
}

#[test]
fn segmented_and_tab_styles_use_shared_state_priority() {
    let mut node = segmented_node();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let segmented = segmented_control_style(&node);
    assert_eq!(segmented.background, Some(PALETTE.surface_disabled));
    assert_eq!(segmented.border, Some(PALETTE.border_disabled));
    assert_eq!(segmented.selected_text, PALETTE.text_disabled);

    node.disabled = false;
    let segmented = segmented_control_style(&node);
    assert_eq!(segmented.state, UiPainterResolvedState::Pressed);
    assert_eq!(segmented.background, Some(PALETTE.surface_pressed));
    assert_eq!(segmented.border, Some(PALETTE.accent));

    let mut tab = tab_node();
    tab.checked = true;
    tab.hovered = true;
    let style = tab_style(&tab);
    assert_eq!(style.state, UiPainterResolvedState::Hovered);
    assert_eq!(style.background, Some(PALETTE.surface_hover));
    assert_eq!(tab_text_color(&tab), PALETTE.text);
}

fn segmented_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchInputSegmented".into(),
        role: "Mount".into(),
        component_role: "".into(),
        value_text: "center".into(),
        options: model_rc(vec![
            SharedString::from("left"),
            SharedString::from("center"),
            SharedString::from("right"),
        ]),
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 8.0,
            width: 150.0,
            height: 30.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn labeled_segmented_node() -> TemplatePaneNodeData {
    let mut node = segmented_node();
    node.label_text = "Segmented Control".into();
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(161, 172, 178);
    node.label_brightness = 0.94;
    node.layout_offset_x = 6.0;
    node.frame = TemplateNodeFrameData {
        x: 12.0,
        y: 4.0,
        width: 150.0,
        height: 48.0,
    };
    node
}

fn tab_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchDrawerTabComponents".into(),
        role: "Mount".into(),
        text: "UI Components".into(),
        checked: true,
        selected: true,
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 4.0,
            width: 150.0,
            height: 40.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn frame_rect(frame: &TemplateNodeFrameData) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
