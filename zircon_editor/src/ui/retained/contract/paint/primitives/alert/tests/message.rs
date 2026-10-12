use super::*;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

#[test]
fn tall_alert_message_uses_runtime_word_wrap() {
    let node = TemplatePaneNodeData {
        text: "Asset import needs validation before opening the selected project.".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 260.0,
        height: 88.0,
    };
    let mut commands = Vec::new();

    push_alert_message(&mut commands, &node, &rect, 28.0, 254.0, &rect, 2, 1.0);

    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0].text_layout_policy,
        HostTextLayoutPolicy::WordWrap
    );
    assert!(commands[0].frame.height > commands[0].line_height);
}
