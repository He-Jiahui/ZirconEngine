use std::collections::BTreeMap;

use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, FloatingWindowLayout, LayoutCommand,
    LayoutManager, MainHostPageLayout, MainPageId, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId};

#[test]
fn default_workbench_layout_seeds_drawers_inside_workbench_activity_window() {
    let layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");
    let activity_windows = layout.activity_windows();
    let workbench_window = activity_windows
        .get(&window_id)
        .expect("default layout should include workbench activity window");

    assert_eq!(workbench_window.descriptor_id.0, "editor.workbench_window");
    assert_eq!(
        workbench_window.activity_drawers.len(),
        ActivityDrawerSlot::ALL.len()
    );
    assert!(workbench_window
        .activity_drawers
        .contains_key(&ActivityDrawerSlot::LeftTop));
    assert!(workbench_window
        .activity_drawers
        .contains_key(&ActivityDrawerSlot::Bottom));
}

#[test]
fn serialized_workbench_layout_rejects_retired_root_drawers() {
    let mut encoded = serde_json::to_value(WorkbenchLayout::default()).unwrap();
    let object = encoded.as_object_mut().expect("workbench layout object");
    let activity_windows = object
        .remove("activity_windows")
        .expect("current layout must serialize activity windows");
    object.insert("drawers".to_string(), activity_windows);

    assert!(serde_json::from_value::<WorkbenchLayout>(encoded).is_err());
}

#[test]
fn serialized_workbench_layout_rejects_missing_activity_window_fields() {
    let encoded = serde_json::to_value(WorkbenchLayout::default()).unwrap();

    for required_field in ["menu_overflow_mode", "region_overrides", "view_overrides"] {
        let mut missing_field = encoded.clone();
        missing_field["activity_windows"]["window:workbench"]
            .as_object_mut()
            .expect("workbench activity window")
            .remove(required_field);

        assert!(serde_json::from_value::<WorkbenchLayout>(missing_field).is_err());
    }
}

#[test]
fn serialized_workbench_layout_rejects_unknown_nested_fields() {
    let encoded = serde_json::to_value(WorkbenchLayout::default()).unwrap();

    let mut unknown_activity_window_field = encoded.clone();
    unknown_activity_window_field["activity_windows"]["window:workbench"]
        .as_object_mut()
        .expect("workbench activity window")
        .insert("legacy_drawers".to_string(), serde_json::json!({}));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_activity_window_field).is_err());

    let mut unknown_main_page_field = encoded;
    unknown_main_page_field["main_pages"][0]["WorkbenchPage"]
        .as_object_mut()
        .expect("workbench main page")
        .insert("legacy_window".to_string(), serde_json::json!(null));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_main_page_field).is_err());

    let mut with_floating_window = WorkbenchLayout::default();
    with_floating_window
        .floating_windows
        .push(FloatingWindowLayout {
            window_id: MainPageId::new("floating:scene"),
            title: "Scene".to_string(),
            workspace: DocumentNode::default(),
            focused_view: None,
            frame: ShellFrame::default(),
        });
    let mut unknown_floating_window_field = serde_json::to_value(with_floating_window).unwrap();
    unknown_floating_window_field["floating_windows"][0]
        .as_object_mut()
        .expect("floating window")
        .insert("legacy_frame".to_string(), serde_json::json!({}));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_floating_window_field).is_err());
}

#[test]
fn activity_drawer_slot_rejects_retired_bottom_aliases() {
    assert_eq!(
        serde_json::from_str::<ActivityDrawerSlot>(r#""Bottom""#).unwrap(),
        ActivityDrawerSlot::Bottom
    );
    for retired in [r#""BottomLeft""#, r#""BottomRight""#] {
        assert!(serde_json::from_str::<ActivityDrawerSlot>(retired).is_err());
    }
}

#[test]
fn drawer_layout_commands_mutate_default_activity_window_drawers() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");

    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: 144.0,
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerMode {
                slot: ActivityDrawerSlot::LeftTop,
                mode: ActivityDrawerMode::Collapsed,
            },
        )
        .unwrap();

    let drawer =
        &layout.activity_windows[&window_id].activity_drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(drawer.extent, 144.0);
    assert_eq!(drawer.mode, ActivityDrawerMode::Collapsed);
}

#[test]
fn drawer_layout_commands_mutate_active_activity_window_drawers() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let asset_page_id = MainPageId::new("asset-browser");
    let asset_window_id = ActivityWindowId::new("window:z_asset_browser");
    let mut asset_drawer = ActivityDrawerLayout::new(ActivityDrawerSlot::LeftTop);
    asset_drawer.extent = 192.0;

    layout.main_pages.push(MainHostPageLayout::WorkbenchPage {
        id: asset_page_id.clone(),
        title: "Asset Browser".to_string(),
        activity_window: asset_window_id.clone(),
    });
    layout.activity_windows.insert(
        asset_window_id.clone(),
        ActivityWindowLayout {
            window_id: asset_window_id.clone(),
            descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
            host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
            activity_drawers: BTreeMap::from([(ActivityDrawerSlot::LeftTop, asset_drawer)]),
            content_workspace: DocumentNode::default(),
            menu_overflow_mode: Default::default(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
    );
    layout.active_main_page = asset_page_id;

    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: 336.0,
            },
        )
        .unwrap();

    assert_eq!(
        layout.activity_windows[&asset_window_id].activity_drawers[&ActivityDrawerSlot::LeftTop]
            .extent,
        336.0
    );
    assert_ne!(
        layout.activity_windows[&ActivityWindowId::workbench()].activity_drawers
            [&ActivityDrawerSlot::LeftTop]
            .extent,
        336.0
    );
}

#[test]
fn drawer_attach_focus_and_close_commands_mutate_default_activity_window_drawers() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::new("window:workbench");
    let hierarchy = ViewInstanceId::new("editor.hierarchy#1");
    let inspector = ViewInstanceId::new("editor.inspector#1");

    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: hierarchy.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: inspector.clone(),
                target: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
            },
        )
        .unwrap();
    manager
        .apply(
            &mut layout,
            LayoutCommand::FocusView {
                instance_id: hierarchy.clone(),
            },
        )
        .unwrap();

    let drawer =
        &layout.activity_windows[&window_id].activity_drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(
        drawer.tab_stack.tabs,
        vec![hierarchy.clone(), inspector.clone()]
    );
    assert_eq!(drawer.tab_stack.active_tab, Some(hierarchy.clone()));
    assert_eq!(drawer.active_view, Some(hierarchy.clone()));

    manager
        .apply(
            &mut layout,
            LayoutCommand::CloseView {
                instance_id: hierarchy.clone(),
            },
        )
        .unwrap();

    let drawer =
        &layout.activity_windows[&window_id].activity_drawers[&ActivityDrawerSlot::LeftTop];
    assert_eq!(drawer.tab_stack.tabs, vec![inspector.clone()]);
    assert_eq!(drawer.active_view, Some(inspector));
}
