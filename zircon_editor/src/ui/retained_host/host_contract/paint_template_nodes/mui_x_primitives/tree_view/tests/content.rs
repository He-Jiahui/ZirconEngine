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
