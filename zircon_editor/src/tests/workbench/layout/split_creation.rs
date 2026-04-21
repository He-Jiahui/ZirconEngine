use crate::ui::workbench::layout::{
    DocumentNode, LayoutCommand, LayoutManager, MainHostPageLayout, MainPageId, SplitAxis,
    SplitPlacement, WorkbenchLayout, WorkspaceTarget,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

#[test]
fn create_split_can_insert_before_target_tabs() {
    let manager = LayoutManager::default();
    let existing = ViewInstanceId::new("editor.scene#1");
    let inserted = ViewInstanceId::new("editor.hierarchy#1");
    let mut layout = WorkbenchLayout::default();

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: existing.clone(),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
        )
        .unwrap();

    manager
        .apply(
            &mut layout,
            LayoutCommand::CreateSplit {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::Before,
                new_instance: inserted.clone(),
            },
        )
        .unwrap();

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };

    let DocumentNode::SplitNode { first, second, .. } = document_workspace else {
        panic!("expected split root");
    };
    let DocumentNode::Tabs(first_tabs) = first.as_ref() else {
        panic!("expected first tabs");
    };
    let DocumentNode::Tabs(second_tabs) = second.as_ref() else {
        panic!("expected second tabs");
    };

    assert_eq!(first_tabs.tabs, vec![inserted]);
    assert_eq!(second_tabs.tabs, vec![existing]);
}
