use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_asset::project::ProjectPaths;
use zircon_runtime::scene::DefaultLevelManager;

use crate::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode,
    EditorProjectDocument, FloatingWindowLayout, MainHostPageLayout, MainPageId, NewProjectDraft,
    NewProjectTemplate, ProjectEditorWorkspace, TabStackLayout, ViewInstanceId, WorkbenchLayout,
};

#[test]
fn editor_project_document_roundtrips_world_and_workspace() {
    let manager = DefaultLevelManager::default();
    let world = manager.create_default_level().snapshot();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon_editor_project_{unique}"));
    let workspace = ProjectEditorWorkspace {
        layout_version: 1,
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::new("main"),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::new("main"),
                title: "Workbench".to_string(),
                document_workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("scene#1")],
                    active_tab: Some(ViewInstanceId::new("scene#1")),
                }),
            }],
            drawers: BTreeMap::from([(
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftTop,
                    tab_stack: TabStackLayout {
                        tabs: vec![ViewInstanceId::new("hierarchy#1")],
                        active_tab: Some(ViewInstanceId::new("hierarchy#1")),
                    },
                    active_view: Some(ViewInstanceId::new("hierarchy#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 240.0,
                    visible: true,
                },
            )]),
            floating_windows: vec![FloatingWindowLayout {
                window_id: MainPageId::new("float#1"),
                title: "Scene".to_string(),
                workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("scene#1")],
                    active_tab: Some(ViewInstanceId::new("scene#1")),
                }),
                focused_view: Some(ViewInstanceId::new("scene#1")),
                frame: crate::ShellFrame::default(),
            }],
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        open_view_instances: Vec::new(),
        active_center_tab: Some(ViewInstanceId::new("scene#1")),
        active_drawers: vec![ActivityDrawerSlot::LeftTop],
    };

    EditorProjectDocument::save_to_path(&root, &world, Some(&workspace)).unwrap();
    let loaded = EditorProjectDocument::load_from_path(&root).unwrap();
    let paths = ProjectPaths::from_root(&root).unwrap();

    assert!(paths.manifest_path().exists());
    assert!(paths
        .assets_root()
        .join("materials")
        .join("default.material.toml")
        .exists());
    assert!(paths.assets_root().join("models").join("cube.obj").exists());

    assert_eq!(loaded.world.nodes().len(), world.nodes().len());
    assert_eq!(
        loaded.editor_workspace.unwrap().workbench.active_main_page,
        MainPageId::new("main")
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn create_renderable_template_scaffolds_directory_project_defaults() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let location = std::env::temp_dir().join(format!("zircon_editor_welcome_{unique}"));
    fs::create_dir_all(&location).unwrap();
    let draft = NewProjectDraft {
        project_name: "WelcomeProject".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created_root = EditorProjectDocument::create_renderable_template(&draft).unwrap();
    let paths = ProjectPaths::from_root(&created_root).unwrap();

    assert!(paths.root().exists());
    assert!(paths.manifest_path().exists());
    assert!(paths
        .assets_root()
        .join("scenes")
        .join("main.scene.toml")
        .exists());
    assert!(paths
        .assets_root()
        .join("materials")
        .join("default.material.toml")
        .exists());
    assert!(paths
        .assets_root()
        .join("shaders")
        .join("pbr.wgsl")
        .exists());
    assert!(paths.library_root().exists());

    let loaded = EditorProjectDocument::load_from_path(&created_root).unwrap();
    assert_eq!(loaded.manifest.name, "WelcomeProject");
    assert_eq!(
        loaded.manifest.default_scene.to_string(),
        "res://scenes/main.scene.toml"
    );
    assert!(!loaded.world.nodes().is_empty());

    let _ = fs::remove_dir_all(&location);
}
