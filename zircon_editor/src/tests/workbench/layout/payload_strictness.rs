use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, WorkbenchLayout,
};

#[test]
fn serialized_workbench_layout_rejects_deep_unknown_fields() {
    let encoded = serde_json::to_value(WorkbenchLayout::default()).unwrap();

    let mut unknown_drawer_field = encoded.clone();
    unknown_drawer_field["activity_windows"]["window:workbench"]["activity_drawers"]["LeftTop"]
        .as_object_mut()
        .expect("left-top drawer")
        .insert("legacy_extent".to_string(), serde_json::json!(260.0));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_drawer_field).is_err());

    let mut unknown_tab_stack_field = encoded.clone();
    unknown_tab_stack_field["activity_windows"]["window:workbench"]["activity_drawers"]["LeftTop"]
        ["tab_stack"]
        .as_object_mut()
        .expect("left-top tab stack")
        .insert("legacy_active".to_string(), serde_json::json!(null));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_tab_stack_field).is_err());

    let mut retired_main_page_workspace = encoded.clone();
    retired_main_page_workspace["main_pages"][0]["WorkbenchPage"]
        .as_object_mut()
        .expect("workbench page")
        .insert(
            "document_workspace".to_string(),
            serde_json::json!({ "Tabs": { "tabs": [], "active_tab": null } }),
        );
    assert!(serde_json::from_value::<WorkbenchLayout>(retired_main_page_workspace).is_err());

    let mut unknown_document_node_field = encoded.clone();
    unknown_document_node_field["activity_windows"]["window:workbench"]["content_workspace"]
        ["Tabs"]
        .as_object_mut()
        .expect("activity-window content tab stack")
        .insert("legacy_tabs".to_string(), serde_json::json!([]));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_document_node_field).is_err());

    let mut unknown_pane_override_field = encoded.clone();
    unknown_pane_override_field["activity_windows"]["window:workbench"]["region_overrides"]
        ["Left"] = serde_json::json!({
        "width": {},
        "height": {},
        "legacy_axis": {}
    });
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_pane_override_field).is_err());

    let mut unknown_axis_override_field = encoded;
    unknown_axis_override_field["activity_windows"]["window:workbench"]["region_overrides"]
        ["Left"] = serde_json::json!({
        "width": { "legacy_min": 0.0 },
        "height": {}
    });
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_axis_override_field).is_err());

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
    let mut unknown_frame_field = serde_json::to_value(with_floating_window).unwrap();
    unknown_frame_field["floating_windows"][0]["frame"]
        .as_object_mut()
        .expect("floating frame")
        .insert("legacy_x".to_string(), serde_json::json!(0.0));
    assert!(serde_json::from_value::<WorkbenchLayout>(unknown_frame_field).is_err());
}
