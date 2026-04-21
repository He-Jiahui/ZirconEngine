use crate::ui::workbench::layout::{
    DocumentNode, LayoutCommand, LayoutManager, MainHostPageLayout, MainPageId, TabInsertionAnchor,
    TabInsertionSide, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

#[test]
fn attach_view_to_document_inserts_after_anchor_and_keeps_it_active() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.scene#1");
    let second = ViewInstanceId::new("editor.game#1");
    let inserted = ViewInstanceId::new("editor.inspector#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: first.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: inserted.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
                anchor: Some(TabInsertionAnchor {
                    target_id: first.clone(),
                    side: TabInsertionSide::After,
                }),
            },
        )
        .unwrap();

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    let DocumentNode::Tabs(stack) = document_workspace else {
        panic!("expected tabs root");
    };

    assert_eq!(stack.tabs, vec![first, inserted.clone(), second]);
    assert_eq!(stack.active_tab.as_ref(), Some(&inserted));
}
