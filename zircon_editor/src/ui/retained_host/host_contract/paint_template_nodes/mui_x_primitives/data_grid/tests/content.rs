use std::rc::Rc;

use super::*;
use crate::ui::retained_host::primitives::{ModelRc, SharedString, VecModel};

fn values(values: &[&str]) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(
        values
            .iter()
            .map(|value| SharedString::from(*value))
            .collect::<Vec<_>>(),
    )))
}

fn node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        text: "Assets".into(),
        options: values(&["Name", "Type", "Status"]),
        collection_items: values(&[
            "selected|Tree.mesh|Mesh|Ready",
            "normal|Rock.mat|Material|Review",
        ]),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn data_grid_content_paints_authored_title_columns_and_rows() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 220.0,
    };
    let mut commands = Vec::new();

    push_data_grid_content(&mut commands, &node(), &rect, &rect, 10, 1.0);

    let texts = commands
        .iter()
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(texts.contains(&"Assets"));
    assert!(texts.contains(&"Name"));
    assert!(texts.contains(&"Tree.mesh"));
    assert!(texts.contains(&"Review"));
}

#[test]
fn data_grid_content_uses_value_text_for_empty_rows() {
    let mut empty = node();
    empty.collection_items = ModelRc::default();
    empty.value_text = "No assets match the current filters".into();
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 220.0,
    };
    let mut commands = Vec::new();

    push_data_grid_content(&mut commands, &empty, &rect, &rect, 10, 1.0);

    assert!(commands
        .iter()
        .any(|command| { command.text.as_deref() == Some("No assets match the current filters") }));
}

#[test]
fn authored_data_grid_title_paint_scales_once_and_fallback_stays_physical() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        current_host_metrics, enter_host_paint_theme_scope,
        host_paint_theme_snapshot_at_scale_for_test,
    };
    for scale in [1.0, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));

        let rect = FrameRect {
            width: 360.0,
            height: 240.0,
            ..Default::default()
        };
        for (font_size, expected) in [
            (12.0, 12.0 * scale),
            (0.0, current_host_metrics().font_body),
        ] {
            let node = TemplatePaneNodeData {
                text: "DPI title".into(),
                font_size,
                ..Default::default()
            };
            let mut commands = Vec::new();
            push_data_grid_content(&mut commands, &node, &rect, &rect, 0, 1.0);
            let title = commands
                .iter()
                .find(|command| command.text.as_deref() == Some("DPI title"))
                .expect("title paint command");
            assert_eq!(title.font_size, expected);
            assert_eq!(
                title.line_height,
                current_host_metrics().line_height(expected).max(expected)
            );
        }
    }
}
