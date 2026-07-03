use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_status_icon_button_style;
use super::super::template_status_control_geometry::{
    status_control_offset_rect, status_icon_button_glyph_rect, status_icon_button_radius,
    workbench_status_metrics,
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
    push_status_icon_surface(
        commands,
        &rect,
        clip,
        order,
        style.background,
        style.border,
        opacity,
    );
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

fn push_status_icon_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    background: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let background = visible_color(background);
    let border = visible_color(border);
    if background.is_none() && border.is_none() {
        return;
    }
    let border_width = if border.is_some() {
        workbench_status_metrics().border_width
    } else {
        0.0
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        background,
        border,
        border_width,
        status_icon_button_radius(),
        opacity,
    ));
}

fn visible_color(color: [u8; 4]) -> Option<[u8; 4]> {
    (color[3] > 0).then_some(color)
}
