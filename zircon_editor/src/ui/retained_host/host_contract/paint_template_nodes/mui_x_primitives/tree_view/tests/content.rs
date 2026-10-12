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

#[test]
fn tree_view_content_paints_authored_title_and_depth_markers() {
    let node = TemplatePaneNodeData {
        text: "Folders".into(),
        collection_items: values(&[
            "expanded|0|Assets",
            "selected|1|Meshes",
            "normal|1|Materials",
        ]),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 220.0,
    };
    let mut commands = Vec::new();

    push_tree_view_content(&mut commands, &node, &rect, &rect, 10, 1.0);

    let texts = commands
        .iter()
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(texts.contains(&"Folders"));
    assert!(texts.iter().any(|text| text.contains("Assets")));
    assert!(texts.iter().any(|text| text.contains("Meshes")));
}

#[test]
fn authored_tree_view_title_paint_scales_once_and_fallback_stays_physical() {
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
            push_tree_view_content(&mut commands, &node, &rect, &rect, 0, 1.0);
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
