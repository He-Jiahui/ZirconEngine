use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::super::style_selector::is_workbench_slider_state_hot;
use super::super::slider_style;
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiPainterResolvedState, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

pub(super) fn slider_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).label_text
}

pub(super) fn slider_accent(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).fill
}

pub(super) fn slider_track_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).track
}

pub(super) fn slider_thumb_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).thumb
}

pub(super) fn slider_thumb_outline_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).thumb_outline
}

pub(super) fn slider_thumb_halo_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    slider_style(node).thumb_halo
}

pub(super) fn slider_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    slider_style(node).state
}

pub(super) fn slider_visual_hot(node: &TemplatePaneNodeData) -> bool {
    is_workbench_slider_state_hot(slider_visual_state(node))
}

pub(super) fn slider_node(control_id: &str, percent: f32) -> TemplatePaneNodeData {
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

pub(super) fn positioned_slider_node(
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

pub(super) fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
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

pub(super) fn changed_pixel_count(
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

pub(super) fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
