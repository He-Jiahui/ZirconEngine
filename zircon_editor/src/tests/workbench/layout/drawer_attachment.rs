use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, LayoutManager, TabInsertionAnchor,
    TabInsertionSide, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

#[test]
fn attach_view_to_same_drawer_reorders_it_to_the_end_and_keeps_it_active() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.project#1");
    let second = ViewInstanceId::new("editor.inspector#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();

    let drawer = layout.drawers.get(&ActivityDrawerSlot::RightTop).unwrap();
    assert_eq!(drawer.tab_stack.tabs, vec![second, first.clone()]);
    assert_eq!(drawer.tab_stack.active_tab.as_ref(), Some(&first));
    assert_eq!(drawer.active_view.as_ref(), Some(&first));
}

#[test]
fn attach_view_to_drawer_inserts_before_anchor_and_keeps_it_active() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.project#1");
    let second = ViewInstanceId::new("editor.inspector#1");
    let inserted = ViewInstanceId::new("editor.console#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: inserted.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: Some(TabInsertionAnchor {
                    target_id: second.clone(),
                    side: TabInsertionSide::Before,
                }),
            },
        )
        .unwrap();

    let drawer = layout
        .drawers
        .get(&ActivityDrawerSlot::RightBottom)
        .unwrap();
    assert_eq!(drawer.tab_stack.tabs, vec![first, inserted.clone(), second]);
    assert_eq!(drawer.tab_stack.active_tab.as_ref(), Some(&inserted));
    assert_eq!(drawer.active_view.as_ref(), Some(&inserted));
}

#[test]
fn drawer_selection_is_normalized_to_one_active_item_after_layout_commands() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let first = ViewInstanceId::new("editor.project#1");
    let second = ViewInstanceId::new("editor.inspector#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: first.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: second.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
                anchor: None,
            },
        )
        .unwrap();

    {
        let drawer = layout
            .activity_windows
            .get_mut(&crate::ui::workbench::layout::ActivityWindowId::workbench())
            .unwrap()
            .activity_drawers
            .get_mut(&ActivityDrawerSlot::RightTop)
            .unwrap();
        drawer.mode = ActivityDrawerMode::Pinned;
        drawer.tab_stack.active_tab = Some(first.clone());
        drawer.active_view = Some(second.clone());
    }

    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::RightTop,
                extent: 320.0,
            },
        )
        .unwrap();

    let drawer = layout.drawers.get(&ActivityDrawerSlot::RightTop).unwrap();
    assert_eq!(drawer.tab_stack.active_tab.as_ref(), Some(&first));
    assert_eq!(
        drawer.active_view.as_ref(),
        drawer.tab_stack.active_tab.as_ref(),
        "drawer selection must have one authoritative active tab or none"
    );
}
