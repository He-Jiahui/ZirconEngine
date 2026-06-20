use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{
    slider_value_label, SLIDER_FONT_SIZE, SLIDER_LINE_HEIGHT,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_value(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    style: &WorkbenchSliderStyle,
    value_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        value_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.value_surface),
        Some(style.value_border),
        1.0,
        4.0,
        opacity,
    ));
    let label = slider_value_label(node, percent);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: value_rect.x + 6.0,
            y: value_rect.y + (value_rect.height - SLIDER_LINE_HEIGHT).max(0.0) * 0.5,
            width: (value_rect.width - 12.0).max(1.0),
            height: SLIDER_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 1,
        label,
        style.value_text,
        SLIDER_FONT_SIZE,
        SLIDER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
