use std::collections::BTreeMap;

use crate::core::project::RecentProjectValidation;
use crate::scene::viewport::SceneViewportChromeSettings;
use crate::ui::workbench::layout::{MainHostPageLayout, MainPageId, WorkbenchLayout};
use crate::ui::workbench::model::{DocumentWorkspaceModel, WorkbenchViewModel};
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
    ViewContentKind,
};
use crate::ui::workbench::startup::{
    EditorSessionMode, NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot,
};
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewKind,
    WorkbenchSlot,
};
use zircon_runtime_interface::math::UVec2;

#[test]
fn welcome_startup_projects_into_exclusive_page_model() {
    let descriptors = vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.welcome"),
            ViewKind::ActivityWindow,
            "Welcome",
        )
        .with_workbench_slot(WorkbenchSlot::ExclusiveMainPage)
        .with_icon_key("welcome"),
    ];
    let welcome_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.welcome#1"),
        descriptor_id: ViewDescriptorId::new("editor.welcome"),
        title: "Welcome".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new("page:welcome")),
    };
    let chrome = EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Default::default(),
            inspector: None,
            status_line: "Welcome".to_string(),
            console_output: "Welcome".into(),
            status_task_progress: None,
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: SceneViewportChromeSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: String::new(),
            session_mode: EditorSessionMode::Welcome,
            welcome: WelcomePaneSnapshot {
                title: "Open or Create".to_string(),
                subtitle: "Recent projects and a renderable empty-project template".to_string(),
                status_message: "No recent project".to_string(),
                browse_supported: false,
                recent_projects: vec![RecentProjectItemSnapshot {
                    display_name: "Broken".to_string(),
                    path: "E:/Missing/Broken".to_string(),
                    validation: RecentProjectValidation::Missing,
                    last_opened_label: "Just now".to_string(),
                    selected: true,
                }],
                form: NewProjectFormSnapshot {
                    project_name: "WelcomeProject".to_string(),
                    location: "E:/Work".to_string(),
                    project_path_preview: "E:/Work/WelcomeProject".to_string(),
                    template_label: "Renderable Empty".to_string(),
                    can_create: true,
                    can_open_existing: true,
                    validation_message: String::new(),
                },
            },
            project_open: false,
            can_undo: false,
            can_redo: false,
            bridge_diagnostics: Default::default(),
        },
        &WorkbenchLayout {
            active_main_page: MainPageId::new("page:welcome"),
            main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:welcome"),
                title: "Welcome".to_string(),
                window_instance: welcome_instance.instance_id.clone(),
            }],
            drawers: BTreeMap::new(),
            activity_windows: Default::default(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![welcome_instance],
        descriptors,
        None,
    );

    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    assert!(!model.drawer_ring.visible);
    assert!(matches!(
        model.document,
        DocumentWorkspaceModel::Exclusive { ref view, .. }
            if view.content_kind == ViewContentKind::Welcome
    ));
    assert_eq!(
        model
            .host_strip
            .breadcrumbs
            .iter()
            .map(|crumb| crumb.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Welcome", "Open or Create"]
    );
}
