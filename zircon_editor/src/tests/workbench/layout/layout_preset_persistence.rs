use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, DocumentLeafLayout, DocumentNode,
    FloatingWindowLayout, MainHostPageLayout, MainPageId, SplitAxis, TabStackLayout,
    WorkbenchLayout,
};
use crate::ui::workbench::view::ViewInstanceId;
use crate::ui::workbench::{
    CenterSplitLayout, CenterSplitNode, LayoutPreset, LayoutPresetName,
    LayoutPresetPersistenceStore, LayoutPresetRestoreFallback, LayoutPresetRestoreResult,
    LayoutPresetScope,
};

#[test]
fn page_user_layout_persistence_roundtrips_exact_split_tree_tabs_and_ratios() {
    let page_id = MainPageId::workbench();
    let scene = ViewInstanceId::new("editor.scene#persisted-layout");
    let material = ViewInstanceId::new("editor.material#persisted-layout");
    let mut layout = WorkbenchLayout::default();

    let document_workspace = layout
        .content_workspace_for_page_mut(&page_id)
        .expect("default page should resolve its activity-window content workspace");
    *document_workspace = DocumentNode::SplitNode {
        node_id: Default::default(),
        axis: SplitAxis::Vertical,
        ratio: 0.65,
        first: Box::new(DocumentNode::tabs(TabStackLayout {
            tabs: vec![scene.clone()],
            active_tab: Some(scene.clone()),
        })),
        second: Box::new(DocumentNode::tabs(TabStackLayout {
            tabs: vec![material.clone()],
            active_tab: Some(material.clone()),
        })),
    };

    let window = layout
        .activity_windows
        .get_mut(&ActivityWindowId::workbench())
        .expect("default activity window");
    window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::LeftTop)
        .expect("left-top drawer")
        .mode = ActivityDrawerMode::Collapsed;
    window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::RightTop)
        .expect("right-top drawer")
        .extent = 444.0;
    window
        .activity_drawers
        .get_mut(&ActivityDrawerSlot::Bottom)
        .expect("bottom drawer")
        .extent = 300.0;
    let scope = LayoutPresetScope::new("artist", page_id.clone());
    let mut store = LayoutPresetPersistenceStore::default();
    let captured = store.persist_layout_snapshot(scope.clone(), LayoutPresetName::Debug, &layout);

    assert_eq!(captured.name, LayoutPresetName::Debug);
    let CenterSplitLayout::Exact { root } = &captured.center_split else {
        panic!("captured user preset should own the exact document tree");
    };
    let CenterSplitNode::Split {
        node_id,
        axis,
        ratio_bits,
        first,
        second,
    } = root
    else {
        panic!("captured root should remain split");
    };
    assert_eq!(
        *node_id,
        layout
            .content_workspace_for_page(&page_id)
            .unwrap()
            .node_id()
    );
    let saved_root_id = *node_id;
    assert_eq!(*axis, SplitAxis::Vertical);
    assert_eq!(f32::from_bits(*ratio_bits), 0.65);
    assert!(matches!(
        first.as_ref(),
        CenterSplitNode::Tabs { node_id, tabs, active_tab }
            if !node_id.is_nil()
                && tabs == &vec![scene.clone()]
                && active_tab.as_ref() == Some(&scene)
    ));
    assert!(matches!(
        second.as_ref(),
        CenterSplitNode::Tabs { node_id, tabs, active_tab }
            if !node_id.is_nil()
                && tabs == &vec![material.clone()]
                && active_tab.as_ref() == Some(&material)
    ));
    assert!(captured
        .drawer_states
        .iter()
        .any(|state| state.slot == ActivityDrawerSlot::LeftTop
            && state.mode == ActivityDrawerMode::Collapsed));
    assert!(captured
        .size_overrides
        .iter()
        .any(
            |override_value| override_value.token.as_str() == "--right-drawer-width"
                && override_value.value == 444
        ));

    let encoded = serde_json::to_string(&store).expect("layout preset store serializes");
    assert!(encoded.contains("editor.scene#persisted-layout"));
    assert!(encoded.contains("editor.material#persisted-layout"));
    let decoded: LayoutPresetPersistenceStore =
        serde_json::from_str(&encoded).expect("layout preset store deserializes");

    let mut restored_layout = WorkbenchLayout::default();
    let floating_view = ViewInstanceId::new("editor.scene#floating-before-preset");
    restored_layout.floating_windows.push(FloatingWindowLayout {
        window_id: MainPageId::new("floating:before-preset"),
        title: "Floating before preset".to_string(),
        workspace: DocumentNode::Tabs(DocumentLeafLayout {
            node_id: saved_root_id,
            tab_stack: TabStackLayout {
                tabs: vec![floating_view.clone()],
                active_tab: Some(floating_view),
            },
        }),
        focused_view: None,
        frame: ShellFrame::default(),
    });
    let restored = decoded.restore_into_layout(&scope, &mut restored_layout);

    assert!(matches!(restored, LayoutPresetRestoreResult::Restored(_)));
    assert_eq!(
        restored_layout
            .content_workspace_for_page(&page_id)
            .unwrap()
            .node_id(),
        saved_root_id
    );
    assert_ne!(
        restored_layout.floating_windows[0].workspace.node_id(),
        saved_root_id
    );
    let displaced_floating_id = restored_layout.floating_windows[0].workspace.node_id();
    assert!(matches!(
        decoded.restore_into_layout(&scope, &mut restored_layout),
        LayoutPresetRestoreResult::Restored(_)
    ));
    assert_eq!(
        restored_layout.floating_windows[0].workspace.node_id(),
        displaced_floating_id,
        "reapplying a preset must retain an already repaired floating node ID"
    );
    assert_eq!(
        restored_layout,
        serde_json::from_str::<WorkbenchLayout>(&serde_json::to_string(&restored_layout).unwrap())
            .unwrap()
    );
    let restored_drawers = restored_layout.active_activity_window_drawers();
    assert_eq!(
        restored_drawers[&ActivityDrawerSlot::LeftTop].mode,
        ActivityDrawerMode::Collapsed
    );
    assert_eq!(
        restored_drawers[&ActivityDrawerSlot::RightTop].extent,
        444.0
    );
    assert_eq!(restored_drawers[&ActivityDrawerSlot::Bottom].extent, 300.0);

    let document_workspace = restored_layout
        .content_workspace_for_page(&page_id)
        .expect("restored page should resolve its activity-window content workspace");
    let DocumentNode::SplitNode {
        axis,
        ratio,
        first,
        second,
        ..
    } = document_workspace
    else {
        panic!("restored layout should rebuild the center split shape");
    };
    assert_eq!(*axis, SplitAxis::Vertical);
    assert_eq!(*ratio, 0.65);
    assert!(matches!(
        first.as_ref(),
        DocumentNode::Tabs(stack)
            if stack.tabs == vec![scene.clone()] && stack.active_tab.as_ref() == Some(&scene)
    ));
    assert!(matches!(
        second.as_ref(),
        DocumentNode::Tabs(stack)
            if stack.tabs == vec![material.clone()]
                && stack.active_tab.as_ref() == Some(&material)
    ));
}

#[test]
fn preset_restore_preserves_saved_id_against_earlier_activity_window() {
    let page_id = MainPageId::workbench();
    let scope = LayoutPresetScope::new("artist", page_id.clone());
    let saved_id = DocumentNode::default().node_id();
    let mut preset = LayoutPreset::focus();
    preset.center_split = CenterSplitLayout::Exact {
        root: CenterSplitNode::Tabs {
            node_id: saved_id,
            tabs: Vec::new(),
            active_tab: None,
        },
    };
    let mut store = LayoutPresetPersistenceStore::default();
    store.persist_layout(scope.clone(), preset);

    let mut layout = WorkbenchLayout::default();
    let earlier_id = ActivityWindowId::new("window:before-workbench");
    assert!(earlier_id < ActivityWindowId::workbench());
    let mut earlier = layout.activity_windows[&ActivityWindowId::workbench()].clone();
    earlier.window_id = earlier_id.clone();
    earlier.content_workspace = DocumentNode::Tabs(DocumentLeafLayout {
        node_id: saved_id,
        tab_stack: TabStackLayout::default(),
    });
    layout.activity_windows.insert(earlier_id.clone(), earlier);
    layout.main_pages.push(MainHostPageLayout::WorkbenchPage {
        id: MainPageId::new("page:before-workbench"),
        title: "Earlier activity window".to_string(),
        activity_window: earlier_id.clone(),
    });

    assert!(matches!(
        store.restore_into_layout(&scope, &mut layout),
        LayoutPresetRestoreResult::Restored(_)
    ));
    assert_eq!(
        layout
            .content_workspace_for_page(&page_id)
            .unwrap()
            .node_id(),
        saved_id,
        "the restored preset owns its saved document ID regardless of window ordering"
    );
    let displaced_id = layout.activity_windows[&earlier_id]
        .content_workspace
        .node_id();
    assert_ne!(displaced_id, saved_id);
    assert!(matches!(
        store.restore_into_layout(&scope, &mut layout),
        LayoutPresetRestoreResult::Restored(_)
    ));
    assert_eq!(
        layout.activity_windows[&earlier_id]
            .content_workspace
            .node_id(),
        displaced_id
    );
    assert_eq!(
        layout,
        serde_json::from_str::<WorkbenchLayout>(&serde_json::to_string(&layout).unwrap()).unwrap()
    );
}

#[test]
fn page_user_layout_restore_is_scoped_and_falls_back_when_missing() {
    let page_id = MainPageId::workbench();
    let artist_scope = LayoutPresetScope::new("artist", page_id.clone());
    let reviewer_scope = LayoutPresetScope::new("reviewer", page_id.clone());
    let mut store = LayoutPresetPersistenceStore::default();

    store.persist_layout(artist_scope.clone(), LayoutPreset::focus());
    store.persist_layout(reviewer_scope.clone(), LayoutPreset::debug());

    assert_eq!(
        store.restore_layout(&artist_scope).preset().name,
        LayoutPresetName::Focus
    );
    assert_eq!(
        store.restore_layout(&reviewer_scope).preset().name,
        LayoutPresetName::Debug
    );
    assert_eq!(
        store
            .restore_layout(&LayoutPresetScope::new(
                "artist",
                MainPageId::new("asset:42")
            ))
            .fallback_reason(),
        Some(&LayoutPresetRestoreFallback::Missing)
    );
}
