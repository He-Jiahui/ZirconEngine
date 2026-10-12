use std::collections::BTreeMap;

use toml::Value;

use super::*;

fn metadata(attributes: &[(&str, Value)]) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "NotificationCenter".into(),
        attributes: attributes
            .iter()
            .map(|(key, value)| ((*key).into(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
        ..UiTemplateNodeMetadata::default()
    }
}

fn render(metadata: &UiTemplateNodeMetadata) -> Vec<UiRenderCommand> {
    notification_center_render_commands(
        UiNodeId::new(1),
        Some(metadata),
        &UiStateFlags::default(),
        None,
        UiFrame::new(0.0, 0.0, 320.0, 180.0),
        None,
        None,
        0,
        1.0,
    )
}

#[test]
fn notification_center_requires_caller_owned_title_and_empty_copy() {
    let metadata = metadata(&[("open", Value::Boolean(true))]);
    let commands = render(&metadata);

    assert!(commands
        .iter()
        .all(|command| command.kind != UiRenderCommandKind::Text));
}

#[test]
fn notification_center_projects_caller_owned_title_and_empty_copy() {
    let metadata = metadata(&[
        ("open", Value::Boolean(true)),
        ("title", Value::String("Activity".into())),
        ("empty_text", Value::String("Nothing to review".into())),
    ]);
    let commands = render(&metadata);

    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Activity")));
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Nothing to review")));
}
