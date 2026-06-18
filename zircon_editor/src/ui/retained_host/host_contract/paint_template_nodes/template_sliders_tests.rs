use super::super::super::data::{FrameRect, TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::super::style_selector::{
    is_workbench_slider_state_hot, WORKBENCH_SLIDER_THUMB as SLIDER_THUMB,
    WORKBENCH_SLIDER_TRACK as SLIDER_TRACK,
    WORKBENCH_SLIDER_TRACK_DISABLED as SLIDER_TRACK_DISABLED,
};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::super::template_slider_geometry::{
    slider_fill_span, slider_label, slider_percent, slider_range_min_percent, slider_thumb_size,
    slider_tick_count, slider_track_rect, slider_value_label, slider_value_rect,
};
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiPainterResolvedState, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

#[test]
fn workbench_slider_recognizes_slider_ids_and_roles() {
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchInputSlider",
        0.75
    )));
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchInputRangeSlider",
        0.8
    )));
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchInputStepsSlider",
        0.86
    )));
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchSliderRoot",
        0.5
    )));
    assert!(!is_workbench_slider(&slider_node("PlainSlider", 0.5)));
}

#[test]
fn declared_workbench_slider_variant_is_recognized_without_workbench_id() {
    let mut node = slider_node("PlainSlider", 0.5);
    node.component_variant = "workbench-slider".into();

    assert!(is_workbench_slider(&node));
}

#[test]
fn workbench_slider_paints_track_fill_thumb_and_value() {
    let bytes = paint_template_nodes_for_test(
        220,
        48,
        model_rc(vec![positioned_slider_node(
            "WorkbenchInputSlider",
            0.75,
            8.0,
            8.0,
            184.0,
            30.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 220, 24, 23), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 220, 126, 23), SLIDER_TRACK);
    assert_eq!(pixel_at(&bytes, 220, 104, 23), SLIDER_THUMB);
    assert_ne!(pixel_at(&bytes, 220, 152, 23), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 220, 157, 16, 28, 16) > 0);
}

#[test]
fn hovered_workbench_slider_paints_thumb_halo() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 160.0, 30.0);
    node.hovered = true;
    let bytes = paint_template_nodes_for_test(190, 48, model_rc(vec![node]));

    assert_ne!(pixel_at(&bytes, 190, 61, 15), [0, 0, 0, 255]);
}

#[test]
fn workbench_slider_uses_shared_selector_for_drop_hover_halo() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 160.0, 30.0);
    node.drop_hovered = true;

    assert_eq!(
        slider_visual_state(&node),
        UiPainterResolvedState::DropHovered
    );
    assert!(slider_visual_hot(&node));

    let bytes = paint_template_nodes_for_test(190, 48, model_rc(vec![node]));
    assert_ne!(pixel_at(&bytes, 190, 61, 15), [0, 0, 0, 255]);
}

#[test]
fn workbench_slider_uses_declared_fill_and_thumb_size() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
    node.value_color = Color::from_rgb_u8(32, 153, 162);
    node.layout_icon_size = 9.0;
    node.button_style = resolved_background([42, 51, 56, 255]);

    assert_eq!(slider_track_color(&node), [42, 51, 56, 255]);
    assert_eq!(slider_accent(&node), [32, 153, 162, 255]);
    assert_eq!(slider_thumb_size(&node), 9.0);

    node.disabled = true;
    assert_eq!(slider_track_color(&node), SLIDER_TRACK_DISABLED);
}

#[test]
fn workbench_slider_uses_declared_thumb_colors() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
    node.icon_color = Color::from_rgb_u8(183, 241, 248);
    node.state_layer_color = Color::from_argb_u8(61, 50, 211, 222);
    node.button_style = ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(42, 177, 188, 51))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    };

    assert_eq!(slider_thumb_color(&node), [183, 241, 248, 255]);
    assert_eq!(slider_thumb_outline_color(&node), [42, 177, 188, 51]);
    assert_eq!(slider_thumb_halo_color(&node), Some([50, 211, 222, 61]));

    let bytes = paint_template_nodes_for_test(220, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 220, 104, 23), [183, 241, 248, 255]);
    assert_ne!(pixel_at(&bytes, 220, 96, 23), [0, 0, 0, 255]);
}

#[test]
fn workbench_slider_uses_declared_label_value_and_track_proportion() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
    node.label_text = "Value".into();
    node.value_text = "0.75".into();
    node.label_color = Color::from_rgb_u8(136, 147, 153);
    node.layout_content_offset_x = -10.0;
    node.layout_first_cell_offset_x = 18.0;

    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 184.0,
        height: 30.0,
    };
    let value_rect = slider_value_rect(&rect);
    let track_rect = slider_track_rect(
        &rect,
        value_rect.as_ref(),
        slider_label(&node).is_some(),
        &node,
    );

    assert_eq!(slider_label(&node).as_deref(), Some("Value"));
    assert_eq!(slider_value_label(&node, 0.75), "0.75");
    assert_eq!(slider_label_color(&node), [136, 147, 153, 255]);
    assert_eq!(track_rect.x, 68.0);
    assert_eq!(track_rect.width, 80.0);
}

#[test]
fn workbench_range_slider_uses_declared_minimum_span() {
    let mut node = positioned_slider_node("WorkbenchInputRangeSlider", 0.8, 8.0, 8.0, 184.0, 46.0);
    node.layout_second_cell_offset_x = 20.0;

    let range_min = slider_range_min_percent(&node).expect("range minimum");
    let (fill_start, fill_end) = slider_fill_span(slider_percent(&node), Some(range_min));

    assert!((range_min - 0.2).abs() < 0.001);
    assert!((fill_start - 0.2).abs() < 0.001);
    assert!((fill_end - 0.8).abs() < 0.001);
}

#[test]
fn workbench_steps_slider_uses_declared_tick_count() {
    let mut node = positioned_slider_node("WorkbenchInputStepsSlider", 0.86, 8.0, 8.0, 184.0, 30.0);
    node.layout_third_cell_offset_x = 5.0;

    assert_eq!(slider_tick_count(&node), Some(5));
    assert_eq!(slider_range_min_percent(&node), None);
}

fn slider_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).label_text
}

fn slider_accent(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).fill
}

fn slider_track_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).track
}

fn slider_thumb_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).thumb
}

fn slider_thumb_outline_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).thumb_outline
}

fn slider_thumb_halo_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    slider_style(node).thumb_halo
}

fn slider_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    slider_style(node).state
}

fn slider_visual_hot(node: &TemplatePaneNodeData) -> bool {
    is_workbench_slider_state_hot(slider_visual_state(node))
}

fn slider_node(control_id: &str, percent: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "RangeField".into(),
        component_role: "range-field".into(),
        value_percent: percent,
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 30.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn positioned_slider_node(
    control_id: &str,
    percent: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..slider_node(control_id, percent)
    }
}

fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                color[0], color[1], color[2], color[3],
            ))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
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
