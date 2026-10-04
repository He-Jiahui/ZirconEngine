use std::collections::BTreeMap;

use toml::Value;

use super::*;

fn metadata(attributes: &[(&str, Value)]) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "CommandPalette".into(),
        attributes: attributes
            .iter()
            .map(|(key, value)| ((*key).into(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
        ..UiTemplateNodeMetadata::default()
    }
}

fn render(metadata: &UiTemplateNodeMetadata) -> Vec<UiRenderCommand> {
    command_palette_render_commands(
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
fn palette_requires_caller_owned_placeholder_and_empty_copy() {
    let metadata = metadata(&[("open", Value::Boolean(true))]);
    let commands = render(&metadata);

    assert!(commands
        .iter()
        .all(|command| command.kind != UiRenderCommandKind::Text));
}

#[test]
fn palette_projects_caller_owned_placeholder_and_empty_copy() {
    let metadata = metadata(&[
        ("open", Value::Boolean(true)),
        ("placeholder", Value::String("Find command".into())),
        ("empty_text", Value::String("No matching command".into())),
    ]);
    let commands = render(&metadata);

    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Find command")));
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("No matching command")));
}
