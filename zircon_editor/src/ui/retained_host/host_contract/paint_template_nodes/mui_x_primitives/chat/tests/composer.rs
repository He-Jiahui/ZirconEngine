use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_chat_composer_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_inset = [10, 11, 12, 255];
    palette.accent = [20, 21, 22, 255];

    assert_eq!(
        chat_composer_colors_from_host(&TemplatePaneNodeData::default(), palette),
        [[10, 11, 12, 255], [20, 21, 22, 255]]
    );
}

#[test]
fn mui_x_chat_composer_paints_projected_value_text_before_send_action() {
    let node = TemplatePaneNodeData {
        value_text: "Review the responsive collapse path".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 220.0,
        height: 44.0,
    };
    let mut commands = Vec::new();

    push_chat_composer(&mut commands, &node, &rect, &rect, 2, 1.0);

    let text = commands
        .iter()
        .find(|command| command.text.is_some())
        .expect("composer text command");
    assert_eq!(
        text.text.as_deref(),
        Some("Review the responsive collapse path")
    );
    assert!(text.frame.x > rect.x);
    assert!(text.frame.right() < rect.right() - rect.height);
    assert!(
        commands
            .iter()
            .position(|command| command.text.is_some())
            .expect("text order")
            < commands
                .iter()
                .position(|command| command.text.is_none() && command.z_index > text.z_index)
                .expect("send action order")
    );
}
