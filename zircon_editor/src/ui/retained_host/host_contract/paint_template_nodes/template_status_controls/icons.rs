use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_status_icon_button_style;
use super::super::template_status_control_geometry::{
    status_control_offset_rect, status_icon_button_glyph_rect, STATUS_ICON_BUTTON_RADIUS,
};
use super::super::template_status_glyphs::{push_status_icon_glyph, StatusIconKind};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_icon_button(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    opacity: f32,
) {
    let rect = status_control_offset_rect(node, rect);
    let style = select_workbench_status_icon_button_style(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.background),
        Some(style.border),
        1.0,
        STATUS_ICON_BUTTON_RADIUS,
        opacity,
    ));
    let glyph = status_icon_button_glyph_rect(&rect);
    push_status_icon_glyph(
        commands,
        &glyph,
        clip,
        order + 2,
        kind,
        style.glyph,
        opacity,
    );
}
