use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{actions, content, identity, layout, style, surface};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dialog_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let kind = match identity::dialog_paint_state(node) {
        identity::DialogPaintState::NotDialog => return false,
        identity::DialogPaintState::Closed => return true,
        identity::DialogPaintState::Open(kind) => kind,
    };

    let rect = layout::paint_rect(rect);
    if !layout::dialog_has_visible_area(&rect) || !layout::frame_is_within(clip, &rect) {
        return true;
    }

    let unavailable = style::dialog_unavailable(node);
    surface::push_dialog_chrome(
        commands,
        node,
        &rect,
        clip,
        order,
        kind,
        unavailable,
        opacity,
    );
    let action_top = if matches!(kind, identity::DialogKind::AlertDialog) {
        None
    } else {
        actions::push_dialog_actions(
            commands,
            node,
            &rect,
            clip,
            order,
            kind,
            unavailable,
            opacity,
        )
    };
    content::push_dialog_content(
        commands,
        node,
        &rect,
        clip,
        order,
        kind,
        unavailable,
        action_top,
        opacity,
    );
    if matches!(kind, identity::DialogKind::AlertDialog) {
        let _ = actions::push_dialog_actions(
            commands,
            node,
            &rect,
            clip,
            order,
            kind,
            unavailable,
            opacity,
        );
    }
    true
}
