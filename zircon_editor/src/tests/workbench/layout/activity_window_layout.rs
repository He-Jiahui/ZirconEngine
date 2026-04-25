use std::collections::BTreeMap;

use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerSlot, ActivityWindowHostMode, ActivityWindowId,
    ActivityWindowLayout, DocumentNode,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

#[test]
fn activity_window_id_roundtrips_as_stable_string() {
    let id = ActivityWindowId::new("window:workbench");

    let encoded = serde_json::to_string(&id).expect("serialize activity window id");
    let decoded: ActivityWindowId =
        serde_json::from_str(&encoded).expect("deserialize activity window id");

    assert_eq!(decoded, id);
    assert_eq!(decoded.as_str(), "window:workbench");
}

#[test]
fn activity_window_host_mode_roundtrips_embedded_and_native_modes() {
    for mode in [
        ActivityWindowHostMode::EmbeddedMainFrame,
        ActivityWindowHostMode::NativeWindowHandle,
    ] {
        let encoded = serde_json::to_string(&mode).expect("serialize activity window host mode");
        let decoded: ActivityWindowHostMode =
            serde_json::from_str(&encoded).expect("deserialize activity window host mode");

        assert_eq!(decoded, mode);
    }
}

#[test]
fn activity_window_layout_owns_drawers_per_window() {
    let mut drawers = BTreeMap::new();
    let mut left_drawer = ActivityDrawerLayout::new(ActivityDrawerSlot::LeftTop);
    left_drawer.active_view = Some(ViewInstanceId::new("editor.hierarchy#1"));
    drawers.insert(ActivityDrawerSlot::LeftTop, left_drawer);

    let layout = ActivityWindowLayout {
        window_id: ActivityWindowId::new("window:workbench"),
        descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
        host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
        activity_drawers: drawers,
        content_workspace: DocumentNode::default(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    let encoded = serde_json::to_string(&layout).expect("serialize activity window layout");
    let decoded: ActivityWindowLayout =
        serde_json::from_str(&encoded).expect("deserialize activity window layout");

    assert_eq!(decoded.window_id.as_str(), "window:workbench");
    assert_eq!(decoded.activity_drawers.len(), 1);
    assert_eq!(
        decoded.activity_drawers[&ActivityDrawerSlot::LeftTop].active_view,
        Some(ViewInstanceId::new("editor.hierarchy#1"))
    );
}
