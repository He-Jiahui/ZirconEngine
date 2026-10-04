use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommandKind, tree::UiTemplateNodeMetadata,
};

use super::*;
use crate::ui::surface::render::resolve::resolve_style;

fn metadata(component: &str, attributes: &[(&str, Value)]) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.into(),
        attributes: attributes
            .iter()
            .map(|(key, value)| ((*key).into(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
        ..UiTemplateNodeMetadata::default()
    }
}

#[test]
fn agent_plan_projects_status_rows_and_progress_without_image_fallbacks() {
    let metadata = metadata(
        "AgentPlan",
        &[
            ("text", Value::String("Agent plan".into())),
            (
                "collection_items",
                Value::Array(vec![
                    Value::String("done|Map ReactBits references".into()),
                    Value::String("active|Build retained ZUI surface".into()),
                ]),
            ),
            ("value", Value::Float(0.5)),
        ],
    );
    let commands = render(
        UiNodeId::new(1),
        &metadata,
        UiFrame::new(0.0, 0.0, 320.0, 160.0),
        None,
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Map ReactBits references")
    }));
    assert!(commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Quad && (command.frame.width - 152.0).abs() < 2.0
    }));
    assert!(commands.iter().all(|command| command.image.is_none()));
}

#[test]
fn approval_keeps_authored_actions_reachable() {
    let metadata = metadata(
        "AgentApproval",
        &[
            ("text", Value::String("Approval required".into())),
            (
                "value_text",
                Value::String("Write the reviewed layout.".into()),
            ),
            (
                "options",
                Value::Array(vec![
                    Value::String("Deny".into()),
                    Value::String("Allow write".into()),
                ]),
            ),
        ],
    );
    let commands = render(
        UiNodeId::new(2),
        &metadata,
        UiFrame::new(0.0, 0.0, 400.0, 136.0),
        None,
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Deny")));
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Allow write")));
}

#[test]
fn tool_call_keeps_inline_emphasis_in_one_rich_text_flow() {
    let metadata = metadata(
        "ToolCalls",
        &[
            ("text", Value::String("Tool calls".into())),
            (
                "collection_items",
                Value::Array(vec![Value::String("running|search_web|ReactBits".into())]),
            ),
        ],
    );
    let commands = render(
        UiNodeId::new(3),
        &metadata,
        UiFrame::new(0.0, 0.0, 360.0, 140.0),
        None,
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );
    let matching = commands
        .iter()
        .filter(|command| command.text.as_deref() == Some("**search_web**  |  ReactBits"))
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1);
    assert_eq!(
        matching[0].style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::MarkdownInlineV1
    );
}

#[test]
fn missing_content_does_not_invent_product_copy() {
    for component in ["AgentPlan", "ToolCalls", "AgentApproval", "AIUsage"] {
        let metadata = metadata(component, &[]);
        let commands = render(
            UiNodeId::new(4),
            &metadata,
            UiFrame::new(0.0, 0.0, 360.0, 140.0),
            None,
            0,
            1.0,
            &resolve_style(Some(&metadata)),
        );
        for forbidden in [
            "Agent plan",
            "Tool calls",
            "Approval required",
            "AI usage",
            "Deny",
            "Allow",
        ] {
            assert!(
                commands
                    .iter()
                    .all(|command| command.text.as_deref() != Some(forbidden)),
                "{component} must not manufacture visible fallback copy {forbidden:?}"
            );
        }
    }
}

#[test]
fn approval_preserves_long_caller_copy_inside_a_narrow_flow() {
    let body =
        "A localized approval description that is intentionally longer than the compact panel width.";
    let frame = UiFrame::new(0.0, 0.0, 180.0, 112.0);
    let metadata = metadata(
        "AgentApproval",
        &[
            ("text", Value::String("Approval required".into())),
            ("value_text", Value::String(body.into())),
            (
                "options",
                Value::Array(vec![
                    Value::String("Deny".into()),
                    Value::String("Allow".into()),
                ]),
            ),
        ],
    );
    let commands = render(
        UiNodeId::new(5),
        &metadata,
        frame,
        Some(frame),
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );

    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some(body)));
    let body_command = commands
        .iter()
        .find(|command| command.text.as_deref() == Some(body))
        .expect("long approval copy command");
    assert_eq!(
        body_command.style.wrap,
        zircon_runtime_interface::ui::surface::UiTextWrap::Word,
        "caller-owned approval copy must remain in the flow text pipeline"
    );
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Allow")));
    assert!(commands.iter().all(|command| {
        command.frame.x >= frame.x
            && command.frame.y >= frame.y
            && command.frame.right() <= frame.right()
            && command.frame.bottom() <= frame.bottom()
    }));
}
