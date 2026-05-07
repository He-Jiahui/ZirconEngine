use std::collections::BTreeMap;

use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, MainHostPageLayout, MainPageId,
    TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use crate::ui::workbench::window_registry::{
    DrawerBinding, DrawerDockPosition, DrawerViewInstance, EditorWindowRegistry, MenuOverflowMode,
    WindowInstance, WindowKind,
};

#[test]
fn window_registry_requires_explicit_drawer_capable_window_binding() {
    let mut registry = EditorWindowRegistry::default();
    let ordinary = ActivityWindowId::new("window:plain");
    registry.register_window(WindowInstance::new(
        ordinary.clone(),
        ViewDescriptorId::new("editor.plain_window"),
        WindowKind::Ordinary,
        "Plain",
        ActivityWindowHostMode::EmbeddedMainFrame,
    ));

    let result = registry.register_drawer_view(DrawerViewInstance::new(
        ViewInstanceId::new("editor.inspector#1"),
        ViewDescriptorId::new("editor.inspector"),
        "Inspector",
        ordinary,
        DrawerDockPosition::RightTop,
    ));

    assert!(result.unwrap_err().contains("is not drawer-capable"));
}

#[test]
fn window_registry_tracks_drawer_selection_and_rebinding() {
    let mut registry = EditorWindowRegistry::default();
    let workbench = ActivityWindowId::new("window:workbench");
    let asset = ActivityWindowId::new("window:asset");
    registry.register_window(WindowInstance::new(
        workbench.clone(),
        ViewDescriptorId::new("editor.workbench_window"),
        WindowKind::DrawerCapable,
        "Workbench",
        ActivityWindowHostMode::EmbeddedMainFrame,
    ));
    registry.register_window(WindowInstance::new(
        asset.clone(),
        ViewDescriptorId::new("editor.asset_window"),
        WindowKind::DrawerCapable,
        "Asset",
        ActivityWindowHostMode::EmbeddedMainFrame,
    ));

    let drawer_id = ViewInstanceId::new("editor.inspector#1");
    registry
        .register_drawer_view(DrawerViewInstance::new(
            drawer_id.clone(),
            ViewDescriptorId::new("editor.inspector"),
            "Inspector",
            workbench.clone(),
            DrawerDockPosition::RightTop,
        ))
        .unwrap();

    assert_eq!(
        registry
            .get_window(&workbench)
            .and_then(|window| window.selected_drawer.as_ref()),
        Some(&drawer_id)
    );

    registry
        .bind_drawer(DrawerBinding::new(
            asset.clone(),
            drawer_id.clone(),
            DrawerDockPosition::Bottom,
        ))
        .unwrap();

    assert!(registry
        .get_window(&workbench)
        .unwrap()
        .drawer_views
        .values()
        .all(Vec::is_empty));
    assert_eq!(
        registry
            .get_drawer_view(&drawer_id)
            .map(|drawer| (&drawer.owner_window, drawer.dock_position)),
        Some((&asset, DrawerDockPosition::Bottom))
    );
}

#[test]
fn window_registry_syncs_only_registered_activity_window_drawers() {
    let layout = sample_layout_with_plain_active_window();
    let instances = vec![ViewInstance {
        instance_id: ViewInstanceId::new("editor.inspector#1"),
        descriptor_id: ViewDescriptorId::new("editor.inspector"),
        title: "Inspector".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
    }];

    let registry = EditorWindowRegistry::sync_from_layout(&layout, &instances);

    assert_eq!(
        registry
            .active_window()
            .map(|window| window.window_id.as_str()),
        Some("window:asset")
    );
    assert_eq!(
        registry
            .get_window(&ActivityWindowId::new("window:asset"))
            .map(|window| window.kind),
        Some(WindowKind::Ordinary)
    );
    assert!(registry.selected_drawer_for_active_window().is_none());
    assert!(registry
        .get_drawer_view(&ViewInstanceId::new("editor.inspector#1"))
        .is_some());
}

#[test]
fn window_registry_syncs_detached_drawer_window_without_reclassifying_plain_floating_windows() {
    let drawer_window = MainPageId::new("drawer-window:editor.inspector:1");
    let document_window = MainPageId::new("window:scene");
    let drawer_view = ViewInstanceId::new("editor.inspector#1");
    let scene_view = ViewInstanceId::new("editor.scene#1");
    let mut layout = sample_layout_with_plain_active_window();
    layout.floating_windows.push(floating_window(
        drawer_window.clone(),
        "Inspector",
        drawer_view.clone(),
    ));
    layout.floating_windows.push(floating_window(
        document_window.clone(),
        "Scene",
        scene_view.clone(),
    ));
    let instances = vec![
        ViewInstance {
            instance_id: drawer_view.clone(),
            descriptor_id: ViewDescriptorId::new("editor.inspector"),
            title: "Inspector".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::FloatingWindow(drawer_window.clone(), vec![]),
        },
        ViewInstance {
            instance_id: scene_view.clone(),
            descriptor_id: ViewDescriptorId::new("editor.scene"),
            title: "Scene".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::FloatingWindow(document_window.clone(), vec![]),
        },
    ];

    let registry = EditorWindowRegistry::sync_from_layout(&layout, &instances);

    assert_eq!(
        registry
            .get_drawer_window(&drawer_window)
            .map(|window| (&window.drawer_view, window.title.as_str())),
        Some((&drawer_view, "Inspector"))
    );
    assert!(registry.get_drawer_window(&document_window).is_none());
    assert_eq!(
        registry
            .get_window(&ActivityWindowId::new(drawer_window.0.clone()))
            .map(|window| window.kind),
        Some(WindowKind::DrawerWindow)
    );
    assert_eq!(
        registry
            .get_drawer_view(&drawer_view)
            .map(|view| view.owner_window.as_str()),
        Some(drawer_window.0.as_str())
    );
}

#[test]
fn window_registry_syncs_collapsed_active_drawer_without_selecting_tab() {
    let layout = sample_layout_with_collapsed_active_drawer();
    let drawer_id = ViewInstanceId::new("editor.inspector#1");
    let instances = vec![ViewInstance {
        instance_id: drawer_id.clone(),
        descriptor_id: ViewDescriptorId::new("editor.inspector"),
        title: "Inspector".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
    }];

    let registry = EditorWindowRegistry::sync_from_layout(&layout, &instances);
    let workbench = ActivityWindowId::new("window:workbench");

    let window = registry
        .get_window(&workbench)
        .expect("workbench window should be registered");
    assert_eq!(window.kind, WindowKind::DrawerCapable);
    assert!(window.selected_drawer.is_none());
    assert!(registry.get_drawer_view(&drawer_id).is_some());
    assert!(registry.selected_drawer_for_active_window().is_none());
}

#[test]
fn window_registry_syncs_active_window_menu_overflow_preference() {
    let mut layout = sample_layout_with_plain_active_window();
    layout
        .default_activity_window_mut()
        .expect("workbench window")
        .menu_overflow_mode = MenuOverflowMode::MultiColumn;

    let registry = EditorWindowRegistry::sync_from_layout(&layout, &[]);

    assert_eq!(
        registry
            .get_window(&ActivityWindowId::workbench())
            .map(|window| window.menu_overflow_mode),
        Some(MenuOverflowMode::MultiColumn)
    );
}

fn sample_layout_with_plain_active_window() -> WorkbenchLayout {
    let workbench = ActivityWindowId::new("window:workbench");
    let asset = ActivityWindowId::new("window:asset");
    let drawer_view = ViewInstanceId::new("editor.inspector#1");
    WorkbenchLayout {
        active_main_page: MainPageId::new("asset"),
        main_pages: vec![
            MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: workbench.clone(),
                document_workspace: DocumentNode::default(),
            },
            MainHostPageLayout::WorkbenchPage {
                id: MainPageId::new("asset"),
                title: "Asset".to_string(),
                activity_window: asset.clone(),
                document_workspace: DocumentNode::default(),
            },
        ],
        drawers: BTreeMap::new(),
        activity_windows: BTreeMap::from([
            (
                workbench.clone(),
                ActivityWindowLayout {
                    window_id: workbench,
                    descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::from([(
                        ActivityDrawerSlot::RightTop,
                        ActivityDrawerLayout {
                            tab_stack: TabStackLayout {
                                tabs: vec![drawer_view.clone()],
                                active_tab: Some(drawer_view.clone()),
                            },
                            active_view: Some(drawer_view.clone()),
                            ..ActivityDrawerLayout::new(ActivityDrawerSlot::RightTop)
                        },
                    )]),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            ),
            (
                asset.clone(),
                ActivityWindowLayout {
                    window_id: asset,
                    descriptor_id: ViewDescriptorId::new("editor.asset_window"),
                    host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                    activity_drawers: BTreeMap::new(),
                    content_workspace: DocumentNode::default(),
                    menu_overflow_mode: Default::default(),
                    region_overrides: BTreeMap::new(),
                    view_overrides: BTreeMap::new(),
                },
            ),
        ]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}

fn sample_layout_with_collapsed_active_drawer() -> WorkbenchLayout {
    let workbench = ActivityWindowId::new("window:workbench");
    let drawer_view = ViewInstanceId::new("editor.inspector#1");
    let mut drawer = ActivityDrawerLayout::new(ActivityDrawerSlot::RightTop);
    drawer.tab_stack.tabs = vec![drawer_view];
    drawer.tab_stack.active_tab = None;
    drawer.active_view = None;
    drawer.mode = ActivityDrawerMode::Collapsed;

    WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            activity_window: workbench.clone(),
            document_workspace: DocumentNode::default(),
        }],
        drawers: BTreeMap::new(),
        activity_windows: BTreeMap::from([(
            workbench.clone(),
            ActivityWindowLayout {
                window_id: workbench,
                descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                activity_drawers: BTreeMap::from([(ActivityDrawerSlot::RightTop, drawer)]),
                content_workspace: DocumentNode::default(),
                menu_overflow_mode: Default::default(),
                region_overrides: BTreeMap::new(),
                view_overrides: BTreeMap::new(),
            },
        )]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}

fn floating_window(
    window_id: MainPageId,
    title: &str,
    instance_id: ViewInstanceId,
) -> crate::ui::workbench::layout::FloatingWindowLayout {
    crate::ui::workbench::layout::FloatingWindowLayout {
        window_id,
        title: title.to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![instance_id.clone()],
            active_tab: Some(instance_id.clone()),
        }),
        focused_view: Some(instance_id),
        frame: crate::ui::workbench::autolayout::ShellFrame::default(),
    }
}
