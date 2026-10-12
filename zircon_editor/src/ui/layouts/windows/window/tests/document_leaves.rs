use super::*;
use crate::ui::workbench::layout::{DocumentNode, MainPageId, TabStackLayout};
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn astra_recursive_projection_uses_each_leaf_active_content_and_ratio() {
    let mut fixture = crate::ui::workbench::fixture::default_preview_fixture();
    let leaf = |instance: &str| {
        DocumentNode::tabs(TabStackLayout {
            tabs: vec![ViewInstanceId::new(instance)],
            active_tab: Some(ViewInstanceId::new(instance)),
        })
    };
    *fixture
        .layout
        .content_workspace_for_page_mut(&MainPageId::workbench())
        .unwrap() = DocumentNode::SplitNode {
        node_id: Default::default(),
        axis: SplitAxis::Horizontal,
        ratio: 0.63,
        first: Box::new(leaf("editor.scene#1")),
        second: Box::new(leaf("editor.game#1")),
    };
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let leaves = project_document_leaves(&model, |tab| {
        let mut pane = blank_pane();
        pane.id = tab.instance_id.0.clone().into();
        pane
    });
    assert_eq!(leaves.len(), 2);
    assert_ne!(leaves[0].node_id, leaves[1].node_id);
    assert_eq!(leaves[0].relative_frame.width, 0.63);
    assert_eq!(leaves[1].relative_frame.x, 0.63);
    assert_eq!(leaves[0].pane.id.as_str(), "editor.scene#1");
    assert_eq!(leaves[1].pane.id.as_str(), "editor.game#1");
}
