use super::*;

#[test]
fn applying_project_workspace_restores_single_instance_registry_state() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let restored_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.hierarchy#restored"),
        descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
        title: "Hierarchy".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
    };
    let workspace = ProjectEditorWorkspace {
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
            }],
            activity_windows: BTreeMap::from([(
                ActivityWindowId::workbench(),
                ActivityWindowLayout {
                    window_id: ActivityWindowId::workbench(),
                    descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::from([(
                        ActivityDrawerSlot::LeftTop,
                        ActivityDrawerLayout {
                            slot: ActivityDrawerSlot::LeftTop,
                            tab_stack: TabStackLayout {
                                tabs: vec![restored_instance.instance_id.clone()],
                                active_tab: Some(restored_instance.instance_id.clone()),
                            },
                            active_view: Some(restored_instance.instance_id.clone()),
                            mode: ActivityDrawerMode::Pinned,
                            extent: 260.0,
                            visible: true,
                        },
                    )]),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            )]),
            floating_windows: Vec::new(),
        },
        open_view_instances: vec![restored_instance.clone()],
        focused_view: None,
        active_drawers: vec![ActivityDrawerSlot::LeftTop],
    };

    manager.apply_project_workspace(Some(workspace)).unwrap();
    let layout = manager.current_layout();
    let left_top = layout
        .active_activity_window_drawers()
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert!(left_top
        .tab_stack
        .tabs
        .contains(&restored_instance.instance_id));
    assert!(!left_top
        .tab_stack
        .tabs
        .contains(&ViewInstanceId::new("editor.hierarchy#1")));
    let reopened = manager
        .open_view(ViewDescriptorId::new("editor.hierarchy"), None)
        .unwrap();

    assert_eq!(reopened, restored_instance.instance_id);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn applying_project_workspace_preserves_builtin_shell_drawers() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project_shell_drawers");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let workspace = ProjectEditorWorkspace {
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
            }],
            activity_windows: BTreeMap::from([(
                ActivityWindowId::workbench(),
                ActivityWindowLayout {
                    window_id: ActivityWindowId::workbench(),
                    descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::from([
                        (
                            ActivityDrawerSlot::LeftTop,
                            ActivityDrawerLayout {
                                slot: ActivityDrawerSlot::LeftTop,
                                tab_stack: TabStackLayout::default(),
                                active_view: None,
                                mode: ActivityDrawerMode::Collapsed,
                                extent: 0.0,
                                visible: false,
                            },
                        ),
                        (
                            ActivityDrawerSlot::Bottom,
                            ActivityDrawerLayout {
                                slot: ActivityDrawerSlot::Bottom,
                                tab_stack: TabStackLayout::default(),
                                active_view: None,
                                mode: ActivityDrawerMode::Collapsed,
                                extent: 0.0,
                                visible: false,
                            },
                        ),
                    ]),
                    content_workspace: DocumentNode::Tabs(TabStackLayout {
                        tabs: vec![
                            ViewInstanceId::new("editor.scene#1"),
                            ViewInstanceId::new("editor.game#1"),
                        ],
                        active_tab: Some(ViewInstanceId::new("editor.scene#1")),
                    }),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            )]),
            floating_windows: Vec::new(),
        },
        open_view_instances: vec![
            ViewInstance {
                instance_id: ViewInstanceId::new("editor.scene#1"),
                descriptor_id: ViewDescriptorId::new("editor.scene"),
                title: "Scene".to_string(),
                serializable_payload: serde_json::Value::Null,
                dirty: false,
                host: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
            ViewInstance {
                instance_id: ViewInstanceId::new("editor.game#1"),
                descriptor_id: ViewDescriptorId::new("editor.game"),
                title: "Game".to_string(),
                serializable_payload: serde_json::Value::Null,
                dirty: false,
                host: ViewHost::Document(MainPageId::workbench(), vec![]),
            },
        ],
        focused_view: Some(ViewInstanceId::new("editor.scene#1")),
        active_drawers: Vec::new(),
    };

    manager.apply_project_workspace(Some(workspace)).unwrap();
    let layout = manager.current_layout();

    let left_top = layout
        .active_activity_window_drawers()
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert!(left_top
        .tab_stack
        .tabs
        .contains(&ViewInstanceId::new("editor.hierarchy#1")));
    assert_eq!(left_top.mode, ActivityDrawerMode::Pinned);
    assert!(left_top.visible);
    assert!(left_top.extent > 0.0);

    let right_top = layout
        .active_activity_window_drawers()
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert_eq!(
        right_top.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.inspector#1")]
    );

    let bottom = layout
        .active_activity_window_drawers()
        .get(&ActivityDrawerSlot::Bottom)
        .expect("bottom drawer");
    assert_eq!(
        bottom.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.console#1"),
            ViewInstanceId::new("editor.runtime_diagnostics#1"),
            ViewInstanceId::new("editor.build_export_desktop#1"),
        ]
    );

    let workbench_window = layout
        .activity_windows
        .get(&ActivityWindowId::workbench())
        .expect("workbench activity window");
    let activity_left_top = workbench_window
        .activity_drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("workbench activity left top drawer");
    assert!(activity_left_top
        .tab_stack
        .tabs
        .contains(&ViewInstanceId::new("editor.hierarchy#1")));
    let activity_bottom = workbench_window
        .activity_drawers
        .get(&ActivityDrawerSlot::Bottom)
        .expect("workbench activity bottom drawer");
    assert_eq!(
        activity_bottom.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.console#1"),
            ViewInstanceId::new("editor.runtime_diagnostics#1"),
            ViewInstanceId::new("editor.build_export_desktop#1"),
        ]
    );
    assert_eq!(activity_bottom.mode, ActivityDrawerMode::Pinned);
    assert!(activity_bottom.visible);
    assert!(activity_bottom.extent > 0.0);

    let instances = manager
        .current_view_instances()
        .into_iter()
        .map(|instance| instance.instance_id)
        .collect::<Vec<_>>();
    assert!(instances.contains(&ViewInstanceId::new("editor.assets#1")));
    assert!(instances.contains(&ViewInstanceId::new("editor.inspector#1")));
    assert!(instances.contains(&ViewInstanceId::new("editor.console#1")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}
