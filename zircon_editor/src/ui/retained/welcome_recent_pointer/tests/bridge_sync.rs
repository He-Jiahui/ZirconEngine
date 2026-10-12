use super::*;

#[test]
fn sync_viewport_preserves_recent_project_paths() {
    let mut bridge = WelcomeRecentPointerBridge::new();
    let project_paths = vec![String::from("E:/ProjectA"), String::from("E:/ProjectB")];

    assert!(!bridge.sync(WelcomeRecentPointerLayout {
        viewport: UiFrame::new(8.0, 12.0, 120.0, 80.0),
        recent_project_paths: project_paths.clone(),
    }));

    assert!(!bridge.sync_viewport(UiFrame::new(8.0, 12.0, 120.0, 80.0)));
    assert_eq!(bridge.layout.recent_project_paths, project_paths);

    assert!(!bridge.sync_viewport(UiFrame::new(8.0, 12.0, 180.0, 80.0)));
    assert_eq!(
        bridge.layout.recent_project_paths,
        vec![String::from("E:/ProjectA"), String::from("E:/ProjectB")]
    );
}
