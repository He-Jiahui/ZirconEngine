use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{
    SLIDER_FONT_SIZE, SLIDER_HORIZONTAL_INSET, SLIDER_LABEL_WIDTH, SLIDER_LINE_HEIGHT,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + SLIDER_HORIZONTAL_INSET,
            y: rect.y + (rect.height - SLIDER_LINE_HEIGHT).max(0.0) * 0.5,
            width: SLIDER_LABEL_WIDTH,
            height: SLIDER_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        style.label_text,
        SLIDER_FONT_SIZE,
        SLIDER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
