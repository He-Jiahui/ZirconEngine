use crate::ui::workbench::layout::{ActivityWindowId, EditorMainFrameLayout};

#[test]
fn editor_main_frame_layout_tracks_only_window_tabs_and_active_window() {
    let layout = EditorMainFrameLayout {
        active_window: ActivityWindowId::new("window:workbench"),
        window_tabs: vec![
            ActivityWindowId::new("window:workbench"),
            ActivityWindowId::new("window:assets"),
        ],
    };

    let encoded = serde_json::to_value(&layout).expect("serialize editor main frame layout");

    assert_eq!(encoded["active_window"], "window:workbench");
    assert_eq!(encoded["window_tabs"][1], "window:assets");
    assert!(encoded.get("drawers").is_none());
    assert!(encoded.get("activity_drawers").is_none());
}

#[test]
fn editor_main_frame_layout_roundtrips_without_drawer_state() {
    let layout = EditorMainFrameLayout {
        active_window: ActivityWindowId::new("window:ui_layout"),
        window_tabs: vec![ActivityWindowId::new("window:ui_layout")],
    };

    let encoded = serde_json::to_string(&layout).expect("serialize editor main frame layout");
    let decoded: EditorMainFrameLayout =
        serde_json::from_str(&encoded).expect("deserialize editor main frame layout");

    assert_eq!(decoded.active_window.as_str(), "window:ui_layout");
    assert_eq!(decoded.window_tabs.len(), 1);
}
