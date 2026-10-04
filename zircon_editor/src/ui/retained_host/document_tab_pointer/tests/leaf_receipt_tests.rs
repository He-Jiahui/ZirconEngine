use super::*;
use crate::ui::workbench::layout::{DocumentNode, MainPageId, SplitAxis, TabStackLayout};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn astra_leaf_tab_receipts_route_independently_and_reuse_unchanged_layout() {
    let mut fixture = crate::ui::workbench::fixture::default_preview_fixture();
    let leaf = |name: &str| {
        DocumentNode::tabs(TabStackLayout {
            tabs: vec![ViewInstanceId::new(name)],
            active_tab: Some(ViewInstanceId::new(name)),
        })
    };
    let first = leaf("editor.scene#1");
    let second = leaf("editor.game#1");
    let first_key = format!("document:{}", first.node_id());
    let second_key = format!("document:{}", second.node_id());
    *fixture
        .layout
        .content_workspace_for_page_mut(&MainPageId::workbench())
        .unwrap() = DocumentNode::SplitNode {
        node_id: Default::default(),
        axis: SplitAxis::Horizontal,
        ratio: 0.63,
        first: Box::new(first),
        second: Box::new(second),
    };
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = HostDocumentTabPointerBridge::default();
    assert!(bridge.sync(build_host_document_tab_pointer_layout(&model)));
    for (key, instance) in [(first_key, "editor.scene#1"), (second_key, "editor.game#1")] {
        let route = bridge
            .handle_activate_click(&key, 0)
            .unwrap()
            .route
            .unwrap();
        assert_eq!(
            bridge.target_for_route(route),
            Some(&ViewInstanceId::new(instance))
        );
        assert!(bridge.handle_activate_click(&key, 1).is_err());
    }
    assert!(!bridge.sync(build_host_document_tab_pointer_layout(&model)));
}
