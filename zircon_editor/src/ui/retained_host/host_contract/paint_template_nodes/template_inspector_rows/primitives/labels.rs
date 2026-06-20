use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::{
    nested_label_rect, INSPECTOR_LABEL_WIDTH, INSPECTOR_ROW_TEXT_Y,
};
use super::super::style::INSPECTOR_LABEL_COLOR;
use super::text::push_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    color: [u8; 4],
    opacity: f32,
) {
    push_text(
        commands,
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y + INSPECTOR_ROW_TEXT_Y,
            width: INSPECTOR_LABEL_WIDTH - 4.0,
            height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
        },
        clip,
        order,
        label,
        color,
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_nested_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    opacity: f32,
) {
    push_text(
        commands,
        nested_label_rect(rect),
        clip,
        order,
        label,
        INSPECTOR_LABEL_COLOR,
        opacity,
    );
}
