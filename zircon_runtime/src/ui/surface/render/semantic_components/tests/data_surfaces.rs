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
fn tree_view_projects_hierarchy_rows_without_icon_images() {
    let metadata = metadata(
        "TreeView",
        &[
            ("text", Value::String("Folders".into())),
            (
                "collection_items",
                Value::Array(vec![
                    Value::String("expanded|0|Assets".into()),
                    Value::String("selected|1|Meshes".into()),
                ]),
            ),
        ],
    );
    let commands = render(
        UiNodeId::new(1),
        &metadata,
        UiFrame::new(0.0, 0.0, 240.0, 220.0),
        None,
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Meshes")));
    assert!(commands.iter().all(|command| command.image.is_none()));
}

#[test]
fn data_grid_projects_headers_and_cells() {
    let metadata = metadata(
        "DataGrid",
        &[
            ("text", Value::String("Assets".into())),
            (
                "options",
                Value::Array(vec![
                    Value::String("Name".into()),
                    Value::String("Type".into()),
                ]),
            ),
            (
                "collection_items",
                Value::Array(vec![Value::String("selected|Oak|Mesh|2 MB|Ready".into())]),
            ),
        ],
    );
    let commands = render(
        UiNodeId::new(2),
        &metadata,
        UiFrame::new(0.0, 0.0, 520.0, 220.0),
        None,
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Name")));
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Oak")));
    assert!(commands
        .iter()
        .all(|command| command.kind != UiRenderCommandKind::Image));
}

#[test]
fn data_grid_uses_the_requested_window_and_semantic_column_alignment() {
    let rows = (0..12)
        .map(|index| Value::String(format!("ready|row-{index}|Mesh|{index} MB|Ready")))
        .collect::<Vec<_>>();
    let metadata = metadata(
        "DataGrid",
        &[
            ("text", Value::String("Assets".into())),
            (
                "options",
                Value::Array(vec![
                    Value::String("Name".into()),
                    Value::String("Type".into()),
                    Value::String("Size".into()),
                    Value::String("Status".into()),
                ]),
            ),
            (
                "column_alignments",
                Value::Array(vec![
                    Value::String("left".into()),
                    Value::String("left".into()),
                    Value::String("right".into()),
                    Value::String("center".into()),
                ]),
            ),
            ("viewport_start", Value::Integer(4)),
            ("visible_limit", Value::Integer(2)),
            ("collection_items", Value::Array(rows)),
        ],
    );
    let frame = UiFrame::new(0.0, 0.0, 520.0, 160.0);
    let commands = render(
        UiNodeId::new(3),
        &metadata,
        frame,
        Some(frame),
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("row-4")));
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("row-5")));
    assert!(commands
        .iter()
        .all(|command| command.text.as_deref() != Some("row-3")));
    assert!(commands
        .iter()
        .all(|command| command.text.as_deref() != Some("row-6")));
    let size = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Size"))
        .expect("Size header");
    let status = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Status"))
        .expect("Status header");
    assert_eq!(
        size.style.text_align,
        zircon_runtime_interface::ui::surface::UiTextAlign::Right
    );
    assert_eq!(
        status.style.text_align,
        zircon_runtime_interface::ui::surface::UiTextAlign::Center
    );
    assert!(commands
        .iter()
        .all(|command| command.frame.bottom() <= frame.bottom()));
}

#[test]
fn empty_collections_render_only_caller_provided_copy() {
    let metadata = metadata(
        "DataGrid",
        &[
            ("options", Value::Array(vec![Value::String("Name".into())])),
            ("empty_text", Value::String("Nothing matches".into())),
        ],
    );
    let commands = render(
        UiNodeId::new(4),
        &metadata,
        UiFrame::new(0.0, 0.0, 96.0, 72.0),
        None,
        0,
        1.0,
        &resolve_style(Some(&metadata)),
    );

    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Nothing matches")));
    assert!(commands
        .iter()
        .all(|command| command.text.as_deref() != Some("No assets")));
    assert!(commands.iter().all(|command| command.frame.right() <= 96.0));
}

#[test]
fn data_grid_preserves_long_caller_value_inside_a_narrow_flow() {
    let long_value =
        "Localized asset label that must wrap or elide without escaping its authored row.";
    let frame = UiFrame::new(0.0, 0.0, 160.0, 96.0);
    let metadata = metadata(
        "DataGrid",
        &[
            ("options", Value::Array(vec![Value::String("Name".into())])),
            (
                "collection_items",
                Value::Array(vec![Value::String(format!("normal|{long_value}"))]),
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
        .any(|command| command.text.as_deref() == Some(long_value)));
    let value_command = commands
        .iter()
        .find(|command| command.text.as_deref() == Some(long_value))
        .expect("long data-grid value command");
    assert_eq!(
        value_command.style.wrap,
        zircon_runtime_interface::ui::surface::UiTextWrap::Word,
        "caller-owned cell copy must remain in the flow text pipeline"
    );
    assert!(commands.iter().all(|command| {
        command.frame.x >= frame.x
            && command.frame.y >= frame.y
            && command.frame.right() <= frame.right()
            && command.frame.bottom() <= frame.bottom()
    }));
}
