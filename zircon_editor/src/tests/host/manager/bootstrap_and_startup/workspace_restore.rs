use super::*;

use crate::scene::viewport::{
    SceneViewportCameraSnapshot, SceneViewportSettings, SceneViewportWorkspaceSessionSnapshot,
    ViewportCameraSnapshot,
};
use crate::ui::host::EditorHostStartupSession;
use crate::ui::retained_host::build_startup_state;
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_viewport_toolbar_pointer_click, BuiltinViewportToolbarTemplateBridge,
};
use crate::ui::retained_host::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout, ViewportToolbarPointerBridge,
};
use crate::ui::workbench::project::EditorProjectDocument;
use std::sync::Arc;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::layout::UiPoint;

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
        scene_viewport_sessions: BTreeMap::new(),
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
                    content_workspace: DocumentNode::tabs(TabStackLayout {
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
        scene_viewport_sessions: BTreeMap::new(),
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
    assert_eq!(activity_bottom.mode, ActivityDrawerMode::Collapsed);
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

#[test]
fn scene_viewport_click_focus_survives_split_workspace_reopen_without_hover_stealing_focus() {
    use crate::tests::editor_event::support::EventRuntimeHarness;
    use zircon_runtime_interface::ui::surface::UiPointerEventKind;

    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_split_scene_focus_restore");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let scene = manager
        .view_instance_id_for_descriptor(&ViewDescriptorId::new("editor.scene"))
        .expect("the default Scene view should exist");
    let game = manager
        .view_instance_id_for_descriptor(&ViewDescriptorId::new("editor.game"))
        .expect("the default Game view should exist");
    let scene_runtime_id = crate::core::editor_event::ViewInstanceId::new(scene.0.clone());
    let mut workspace = manager.project_workspace();
    *workspace
        .workbench
        .content_workspace_for_page_mut(&MainPageId::workbench())
        .expect("the default workbench page should exist") = DocumentNode::SplitNode {
        node_id: Default::default(),
        axis: crate::ui::workbench::layout::SplitAxis::Horizontal,
        ratio: 0.63,
        first: Box::new(DocumentNode::tabs(TabStackLayout {
            tabs: vec![scene.clone()],
            active_tab: Some(scene.clone()),
        })),
        second: Box::new(DocumentNode::tabs(TabStackLayout {
            tabs: vec![game.clone()],
            active_tab: Some(game.clone()),
        })),
    };
    for instance in &mut workspace.open_view_instances {
        if instance.instance_id == scene {
            instance.host = ViewHost::Document(MainPageId::workbench(), vec![0]);
        } else if instance.instance_id == game {
            instance.host = ViewHost::Document(MainPageId::workbench(), vec![1]);
        }
    }
    workspace.focused_view = Some(game.clone());
    manager.apply_project_workspace(Some(workspace)).unwrap();

    let unfocused_workspace = manager.project_workspace();
    for event_kind in [
        UiPointerEventKind::Move,
        UiPointerEventKind::Up,
        UiPointerEventKind::Cancel,
    ] {
        harness
            .runtime
            .route_scene_viewport_pointer(scene_runtime_id.clone(), event_kind);
        assert_eq!(manager.project_workspace(), unfocused_workspace);
    }

    assert!(harness
        .runtime
        .route_scene_viewport_pointer(scene_runtime_id.clone(), UiPointerEventKind::Down));
    let saved = manager.project_workspace();
    assert_eq!(saved.focused_view, Some(scene.clone()));
    let saved_json = serde_json::to_vec(&saved).unwrap();

    manager.focus_view(&game).unwrap();
    let reopened: ProjectEditorWorkspace = serde_json::from_slice(&saved_json).unwrap();
    manager.apply_project_workspace(Some(reopened)).unwrap();
    assert_eq!(manager.project_workspace().focused_view, Some(scene));
    assert_eq!(manager.current_layout(), saved.workbench);
}

#[test]
fn serialized_startup_split_scene_leaves_seed_before_retained_toolbar_dispatch() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_serialized_split_startup");
    let project_root = unique_temp_dir("zircon_editor_serialized_split_startup_project");
    create_project_with_default_world(&project_root);

    let mut project = ProjectManager::open(&project_root).unwrap();
    project.scan_and_import().unwrap();
    let document = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let left = ViewInstanceId::new("editor.scene#serialized-left");
    let right = ViewInstanceId::new("editor.scene#serialized-right");
    let left_core = crate::core::editor_event::ViewInstanceId::new(left.0.clone());
    let right_core = crate::core::editor_event::ViewInstanceId::new(right.0.clone());

    let mut left_camera = ViewportCameraSnapshot::default();
    left_camera.transform.translation.x = 14.0;
    let mut right_camera = ViewportCameraSnapshot::default();
    right_camera.transform.translation.y = -9.0;
    let mut right_settings = SceneViewportSettings::default();
    right_settings.projection_mode =
        zircon_runtime::core::framework::render::ProjectionMode::Orthographic;
    let mut workbench = WorkbenchLayout::default();
    *workbench
        .content_workspace_for_page_mut(&MainPageId::workbench())
        .expect("default workbench page") = DocumentNode::SplitNode {
        node_id: Default::default(),
        axis: crate::ui::workbench::layout::SplitAxis::Horizontal,
        ratio: 0.42,
        first: Box::new(DocumentNode::tabs(TabStackLayout {
            tabs: vec![left.clone()],
            active_tab: Some(left.clone()),
        })),
        second: Box::new(DocumentNode::tabs(TabStackLayout {
            tabs: vec![right.clone()],
            active_tab: Some(right.clone()),
        })),
    };
    let workspace = ProjectEditorWorkspace {
        workbench,
        open_view_instances: vec![
            ViewInstance {
                instance_id: left.clone(),
                descriptor_id: ViewDescriptorId::new("editor.scene"),
                title: "Serialized Scene Left".to_string(),
                serializable_payload: serde_json::Value::Null,
                dirty: false,
                host: ViewHost::Document(MainPageId::workbench(), vec![0]),
            },
            ViewInstance {
                instance_id: right.clone(),
                descriptor_id: ViewDescriptorId::new("editor.scene"),
                title: "Serialized Scene Right".to_string(),
                serializable_payload: serde_json::Value::Null,
                dirty: false,
                host: ViewHost::Document(MainPageId::workbench(), vec![1]),
            },
        ],
        focused_view: Some(right.clone()),
        active_drawers: Vec::new(),
        scene_viewport_sessions: BTreeMap::from([
            (
                left.clone(),
                SceneViewportWorkspaceSessionSnapshot {
                    settings: SceneViewportSettings::default(),
                    pivot_mode: crate::scene::viewport::PivotMode::Primary,
                    orbit_target: zircon_runtime_interface::math::Vec3::new(1.0, 0.0, 0.0),
                    camera: Some(SceneViewportCameraSnapshot::from(&left_camera)),
                },
            ),
            (
                right.clone(),
                SceneViewportWorkspaceSessionSnapshot {
                    settings: right_settings,
                    pivot_mode: crate::scene::viewport::PivotMode::Centroid,
                    orbit_target: zircon_runtime_interface::math::Vec3::new(-1.0, 0.0, 0.0),
                    camera: Some(SceneViewportCameraSnapshot::from(&right_camera)),
                },
            ),
        ]),
    };
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &document.world,
        Some(&workspace),
    )
    .unwrap();
    drop(document);
    drop(project);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut startup_session = manager.open_project_and_remember(&project_root).unwrap();
    assert!(
        startup_session
            .project
            .as_ref()
            .and_then(|project| project.editor_workspace.as_ref())
            .is_some(),
        "normal ProjectManager open must decode the serialized workspace before startup"
    );
    let state = build_startup_state(
        manager.as_ref(),
        &mut startup_session,
        UVec2::new(1280, 720),
    )
    .unwrap();
    let startup =
        EditorHostStartupSession::from_parts(startup_session, state, Arc::clone(&manager)).unwrap();
    let controller = startup.controller();

    let restored = controller.scene_viewport_workspace_sessions();
    assert_eq!(
        restored.len(),
        2,
        "both serialized Scene leaves seed before retain"
    );
    assert!(restored.contains_key(&left_core));
    assert!(restored.contains_key(&right_core));
    assert_eq!(manager.current_focused_view(), Some(right.clone()));
    assert_ne!(
        restored[&left_core].camera, restored[&right_core].camera,
        "serialized leaves retain distinct camera state"
    );

    let layout = manager.current_layout();
    let workspace = layout
        .content_workspace_for_page(&MainPageId::workbench())
        .expect("serialized split workspace");
    let (left_surface_key, right_surface_key) = match workspace {
        DocumentNode::SplitNode {
            first: left_node,
            second: right_node,
            ratio,
            ..
        } => (
            {
                assert_eq!(*ratio, 0.42, "serialized split ratio must survive startup");
                format!("document:{}", left_node.node_id())
            },
            format!("document:{}", right_node.node_id()),
        ),
        _ => panic!("serialized startup must retain the split document topology"),
    };
    let shell_presentation_source =
        include_str!("../../../../ui/layouts/windows/workbench_host_window/shell_presentation.rs");
    let host_scene_source =
        include_str!("../../../../ui/layouts/windows/workbench_host_window/host_data.rs");
    assert!(shell_presentation_source.contains("project_document_leaves"));
    assert!(shell_presentation_source.contains("retained_scene_data"));
    assert!(host_scene_source.contains("struct HostWindowSceneData"));
    assert!(
        shell_presentation_source.contains("retained_scene_data")
            && shell_presentation_source.contains("ShellPresentation"),
        "startup restore must feed the retained shell presentation before its first frame"
    );
    let template_bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout([
        left_surface_key.as_str(),
        right_surface_key.as_str(),
    ]));
    let before_commands = controller.scene_viewport_workspace_sessions();
    let before_journal = controller.journal().records().len();
    dispatch_shared_viewport_toolbar_pointer_click(
        controller,
        &template_bridge,
        &mut pointer_bridge,
        &left_surface_key,
        &left_core,
        "align.pos_x",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    )
    .unwrap();
    let after_left_command = controller.scene_viewport_workspace_sessions();
    assert_ne!(
        after_left_command[&left_core].camera, before_commands[&left_core].camera,
        "left toolbar callback must mutate only its restored leaf"
    );
    assert_eq!(
        after_left_command[&right_core], before_commands[&right_core],
        "left toolbar callback must preserve the focused right leaf state"
    );
    assert!(controller.journal().records().len() > before_journal);
    let left_command_journal = controller.journal().records().len();
    dispatch_shared_viewport_toolbar_pointer_click(
        controller,
        &template_bridge,
        &mut pointer_bridge,
        &right_surface_key,
        &right_core,
        "align.neg_z",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    )
    .unwrap();
    let after_right_command = controller.scene_viewport_workspace_sessions();
    assert_eq!(
        after_right_command[&left_core], after_left_command[&left_core],
        "right toolbar callback must preserve the independently routed left leaf"
    );
    assert_ne!(
        after_right_command[&right_core].camera, after_left_command[&right_core].camera,
        "right toolbar callback must mutate the right leaf"
    );
    assert!(controller.journal().records().len() > left_command_journal);
    assert_eq!(manager.current_focused_view(), Some(right.clone()));

    controller.retain_scene_viewports(&std::collections::BTreeSet::from([right_core.clone()]));
    let before_stale = controller.scene_viewport_workspace_sessions();
    let before_journal = controller.journal().records().len();
    let stale = dispatch_shared_viewport_toolbar_pointer_click(
        controller,
        &template_bridge,
        &mut pointer_bridge,
        &left_surface_key,
        &left_core,
        "display.cycle",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    );
    assert!(stale.unwrap_err().contains("stale"));
    assert_eq!(controller.scene_viewport_workspace_sessions(), before_stale);
    assert_eq!(controller.journal().records().len(), before_journal);
    assert_eq!(manager.current_focused_view(), Some(right));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}
