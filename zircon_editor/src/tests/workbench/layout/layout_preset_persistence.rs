use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, DocumentNode, MainHostPageLayout,
    MainPageId, SplitAxis, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::view::ViewInstanceId;
use crate::ui::workbench::{
    CenterSplitLayout, LAYOUT_PRESET_PERSISTENCE_VERSION, LayoutPreset, LayoutPresetName,
    LayoutPresetPersistenceStore, LayoutPresetRestoreFallback, LayoutPresetRestoreResult,
    LayoutPresetScope, PersistedLayoutPreset,
};

#[test]
fn page_user_layout_persistence_roundtrips_drawers_widths_and_split_without_view_payloads() {
    let page_id = MainPageId::workbench();
    let scene = ViewInstanceId::new("editor.scene#persisted-layout");
    let material = ViewInstanceId::new("editor.material#persisted-layout");
    let mut layout = WorkbenchLayout::default();

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &mut layout.main_pages[0]
    else {
        panic!("default layout should expose a workbench page");
    };
    *document_workspace = DocumentNode::SplitNode {
        axis: SplitAxis::Vertical,
        ratio: 0.65,
        first: Box::new(DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene.clone()],
            active_tab: Some(scene.clone()),
        })),
        second: Box::new(DocumentNode::Tabs(TabStackLayout {
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
    layout.sync_legacy_drawers_from_active_activity_window();

    let scope = LayoutPresetScope::new("artist", page_id.clone());
    let mut store = LayoutPresetPersistenceStore::default();
    let captured = store.persist_layout_snapshot(scope.clone(), LayoutPresetName::Debug, &layout);

    assert_eq!(captured.name, LayoutPresetName::Debug);
    assert_eq!(
        captured.center_split,
        CenterSplitLayout::Split {
            axis: SplitAxis::Vertical,
            panes: 2
        }
    );
    assert!(
        captured
            .drawer_states
            .iter()
            .any(|state| state.slot == ActivityDrawerSlot::LeftTop
                && state.mode == ActivityDrawerMode::Collapsed)
    );
    assert!(
        captured
            .size_overrides
            .iter()
            .any(
                |override_value| override_value.token.as_str() == "--right-drawer-width"
                    && override_value.value == 444
            )
    );

    let encoded = serde_json::to_string(&store).expect("layout preset store serializes");
    assert!(!encoded.contains("editor.scene#persisted-layout"));
    assert!(!encoded.contains("editor.material#persisted-layout"));
    let decoded: LayoutPresetPersistenceStore =
        serde_json::from_str(&encoded).expect("layout preset store deserializes");

    let mut restored_layout = WorkbenchLayout::default();
    let restored = decoded.restore_into_layout(&scope, &mut restored_layout);

    assert!(matches!(restored, LayoutPresetRestoreResult::Restored(_)));
    assert_eq!(
        restored_layout.drawers[&ActivityDrawerSlot::LeftTop].mode,
        ActivityDrawerMode::Collapsed
    );
    assert_eq!(
        restored_layout.drawers[&ActivityDrawerSlot::RightTop].extent,
        444.0
    );
    assert_eq!(
        restored_layout.drawers[&ActivityDrawerSlot::Bottom].extent,
        300.0
    );

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &restored_layout.main_pages[0]
    else {
        panic!("default layout should expose a workbench page");
    };
    let DocumentNode::SplitNode {
        axis,
        first,
        second,
        ..
    } = document_workspace
    else {
        panic!("restored layout should rebuild the center split shape");
    };
    assert_eq!(*axis, SplitAxis::Vertical);
    assert!(matches!(first.as_ref(), DocumentNode::Tabs(_)));
    assert!(matches!(second.as_ref(), DocumentNode::Tabs(_)));
}

#[test]
fn page_user_layout_restore_is_scoped_and_falls_back_on_version_mismatch() {
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

    let stale_scope = LayoutPresetScope::new("artist", MainPageId::new("scene:stale"));
    store.insert_persisted(
        stale_scope.clone(),
        PersistedLayoutPreset {
            format_version: LAYOUT_PRESET_PERSISTENCE_VERSION - 1,
            preset: LayoutPreset::debug(),
        },
    );

    let restored = store.restore_layout(&stale_scope);
    assert_eq!(
        restored.fallback_reason(),
        Some(&LayoutPresetRestoreFallback::VersionMismatch {
            stored_version: LAYOUT_PRESET_PERSISTENCE_VERSION - 1,
            expected_version: LAYOUT_PRESET_PERSISTENCE_VERSION
        })
    );
    assert_eq!(restored.preset().name, LayoutPresetName::Authoring);
}
