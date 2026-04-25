use std::collections::BTreeMap;

use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::workbench::layout::{MainHostPageLayout, MainPageId, WorkbenchLayout};
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, MainPageSnapshot,
    ProjectOverviewSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    ViewKind,
};

#[test]
fn chrome_builder_marks_exclusive_activity_window_pages() {
    let asset_browser = ViewInstance {
        instance_id: ViewInstanceId::new("editor.asset_browser#1"),
        descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
        title: "Asset Browser".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new("asset-browser")),
    };
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::new("asset-browser"),
        main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
            id: MainPageId::new("asset-browser"),
            title: "Asset Browser".to_string(),
            window_instance: asset_browser.instance_id.clone(),
        }],
        drawers: BTreeMap::new(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let descriptors = vec![ViewDescriptor::new(
        ViewDescriptorId::new("editor.asset_browser"),
        ViewKind::ActivityWindow,
        "Asset Browser",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)];

    let chrome = EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Vec::new(),
            inspector: None,
            status_line: "Ready".to_string(),
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: SceneViewportSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: String::new(),
            session_mode: EditorSessionMode::Welcome,
            welcome: WelcomePaneSnapshot::default(),
            project_open: false,
            can_undo: false,
            can_redo: false,
        },
        &layout,
        vec![asset_browser.clone()],
        descriptors,
    );

    assert_eq!(
        chrome.workbench.active_main_page,
        MainPageId::new("asset-browser")
    );
    let MainPageSnapshot::Exclusive { view, .. } = &chrome.workbench.main_pages[0] else {
        panic!("expected exclusive page");
    };
    assert_eq!(view.instance_id, asset_browser.instance_id);
    assert!(!view.placeholder);
}
