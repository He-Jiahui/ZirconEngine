use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::select_workbench_toast_style;
use super::super::layout::{
    toast_close_rect, toast_has_action, toast_icon_rect, TOAST_FONT_SIZE, TOAST_LINE_HEIGHT,
};
use super::action::push_toast_action;
use super::icon::{push_toast_status_mark, toast_status_mark_size};
use super::surface::push_toast_surface;
use super::text::push_toast_text;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_toast(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = select_workbench_toast_style(node);
    push_toast_surface(
        commands,
        rect,
        clip,
        order,
        style.surface,
        style.border,
        opacity,
    );

    let icon = toast_icon_rect(rect, toast_status_mark_size(node));
    push_toast_status_mark(commands, &icon, clip, order + 1, style.mark, opacity);

    let has_action = toast_has_action(rect);
    let close = toast_close_rect(rect);
    push_toast_text(
        commands,
        node,
        rect,
        &icon,
        &close,
        clip,
        order + 2,
        style.text,
        has_action,
        opacity,
    );

    if has_action {
        push_toast_action(
            commands,
            rect,
            &close,
            clip,
            order + 2,
            style.action,
            style.close,
            TOAST_FONT_SIZE,
            TOAST_LINE_HEIGHT,
            opacity,
        );
    }
}
