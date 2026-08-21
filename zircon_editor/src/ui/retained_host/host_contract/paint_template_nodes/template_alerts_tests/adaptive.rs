use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::super::push_alert_commands;
use super::support::positioned_alert_node;

#[test]
fn degenerate_alert_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node("WorkbenchInfoAlert", "Info", "info", 8.0, 6.0, 0.0, 32.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_inline_alert_omits_mark_and_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node("WorkbenchInfoAlert", "Info", "info", 8.0, 6.0, 1.0, 32.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert_eq!(commands.len(), 1, "only the inline surface should remain");
    assert!(commands.iter().all(|command| command.text.is_none()));
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn feedback_marks_require_an_explicit_icon_and_reclaim_text_width() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 180.0,
        height: 32.0,
    };

    let inline_with_icon = positioned_alert_node(
        "WorkbenchInfoAlert",
        "Information",
        "info",
        rect.x,
        rect.y,
        rect.width,
        rect.height,
    );
    let mut inline_without_icon = inline_with_icon.clone();
    inline_without_icon.icon_name = "".into();
    let inline_with_icon_commands = alert_commands(&inline_with_icon, &rect);
    let inline_without_icon_commands = alert_commands(&inline_without_icon, &rect);

    assert!(
        inline_without_icon_commands
            .iter()
            .all(|command| command.z_index != 5),
        "an inline alert without an icon declaration must not emit a status mark"
    );
    assert!(
        first_text_x(&inline_without_icon_commands) < first_text_x(&inline_with_icon_commands),
        "an inline alert without a mark should reclaim its text width"
    );

    let toast_with_icon = positioned_alert_node(
        "WorkbenchToastRoot",
        "Operation completed",
        "success",
        rect.x,
        rect.y,
        rect.width,
        rect.height,
    );
    let mut toast_without_icon = toast_with_icon.clone();
    toast_without_icon.icon_name = "".into();
    let toast_with_icon_commands = alert_commands(&toast_with_icon, &rect);
    let toast_without_icon_commands = alert_commands(&toast_without_icon, &rect);

    assert!(
        toast_without_icon_commands
            .iter()
            .all(|command| command.z_index != 5),
        "a toast without an icon declaration must not emit a status mark"
    );
    assert!(
        first_text_x(&toast_without_icon_commands) < first_text_x(&toast_with_icon_commands),
        "a toast without a mark should reclaim its text width"
    );
}

#[test]
fn narrow_toast_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed",
            "success",
            8.0,
            6.0,
            24.0,
            28.0,
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn short_toast_omits_text_action_and_marks() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 120.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed",
            "success",
            8.0,
            6.0,
            120.0,
            4.0,
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert_eq!(commands.len(), 1, "only the toast surface should remain");
    assert!(commands.iter().all(|command| command.text.is_none()));
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn alert_outside_its_clip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 9.0,
        y: 6.0,
        width: 120.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 120.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node("WorkbenchInfoAlert", "Info", "info", 9.0, 6.0, 120.0, 32.0),
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

fn alert_commands(
    node: &crate::ui::retained_host::host_contract::data::TemplatePaneNodeData,
    rect: &FrameRect,
) -> Vec<HostPaintCommand> {
    let mut commands = Vec::new();
    assert!(push_alert_commands(&mut commands, node, rect, rect, 4, 1.0));
    commands
}

fn first_text_x(commands: &[HostPaintCommand]) -> f32 {
    commands
        .iter()
        .find_map(|command| command.text.as_ref().map(|_| command.frame.x))
        .expect("feedback fixture should emit a text command")
}
