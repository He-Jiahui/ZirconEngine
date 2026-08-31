use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    project::{ProjectManager, ProjectPaths},
    AssetUri,
};
use zircon_runtime::core::resource::ResourceState;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, HistorySaveMarkOutcome,
};
use crate::core::editing::selection::SceneSelection;
use crate::core::project::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, SceneCreateRequest,
};
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, DocumentNode,
    FloatingWindowLayout, MainHostPageLayout, MainPageId, TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::project::{
    EditorProjectDocument, ProjectEditorWorkspace, ProjectSettingsLoadState,
};
use crate::ui::workbench::view::ViewInstanceId;

#[test]
fn f3_project_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = unique_mvp_project_root("physical-root");
    let executable = std::env::current_exe().expect("locate the F3 test executable");
    let binary_directory = executable
        .parent()
        .expect("F3 test executable must have a parent directory");
    let resolved_binary_directory =
        ProjectPaths::resolve_existing(binary_directory).expect("resolve F3 test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "F3 project fixture output must retain the test binary's physical output root"
    );
}

#[test]
fn editor_project_document_roundtrips_world_and_workspace() {
    const TRANSFORM_X_DELTA: f32 = 4.25;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("editor-project-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let initial = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let cube = initial
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("renderable project must retain the template Cube")
        .clone();
    let camera = initial
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Camera")
        .expect("renderable project must retain the template Camera")
        .clone();
    let sun = initial
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Sun")
        .expect("renderable project must retain the template Sun")
        .clone();
    let workspace = ProjectEditorWorkspace {
        workbench: {
            let mut layout = WorkbenchLayout::default();
            layout.active_main_page = MainPageId::new("main");
            layout.main_pages = vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::new("main"),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
            }];
            let default_window = layout
                .default_activity_window_mut()
                .expect("default workbench window");
            default_window.content_workspace = DocumentNode::Tabs(TabStackLayout {
                tabs: vec![ViewInstanceId::new("scene#1")],
                active_tab: Some(ViewInstanceId::new("scene#1")),
            });
            default_window.activity_drawers = BTreeMap::from([(
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
            )]);
            layout.floating_windows = vec![FloatingWindowLayout {
                window_id: MainPageId::new("float#1"),
                title: "Scene".to_string(),
                workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("scene#1")],
                    active_tab: Some(ViewInstanceId::new("scene#1")),
                }),
                focused_view: Some(ViewInstanceId::new("scene#1")),
                frame: ShellFrame::default(),
            }];
            layout
        },
        open_view_instances: Vec::new(),
        focused_view: Some(ViewInstanceId::new("scene#1")),
        active_drawers: vec![ActivityDrawerSlot::LeftTop],
    };

    let level =
        DefaultLevelManager::default().create_level(initial.world, LevelMetadata::default());
    let mut context = CoreEditContext::default();
    context
        .bind_scene(
            level.clone(),
            SceneSelection::new(vec![cube.id], Some(cube.id)),
        )
        .unwrap();
    let transactions = EditorTransactionEngine::new(context);
    let mut expected_transform = cube.transform.clone();
    expected_transform.translation.x += TRANSFORM_X_DELTA;
    let command = level
        .with_world(|scene| {
            EditorCommand::set_transform(scene, cube.id, expected_transform.clone())
        })
        .unwrap()
        .expect("the fixed F3 transform delta must create an editor command");
    let mut scope = transactions
        .begin("Persist F3 Cube transform", HistoryContextId::Global)
        .unwrap();
    scope.push(command).unwrap();
    scope.commit().unwrap();
    assert!(transactions.is_dirty(HistoryContextId::Global).unwrap());

    let save_token = transactions
        .capture_save_token(HistoryContextId::Global)
        .unwrap();
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &level.snapshot(),
        Some(&workspace),
    )
    .unwrap();
    assert_eq!(
        transactions
            .mark_saved_if_unchanged(HistoryContextId::Global, save_token)
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert!(!transactions.is_dirty(HistoryContextId::Global).unwrap());
    drop(transactions);
    drop(level);
    drop(project);

    let mut reopened_project = ProjectManager::open(&root).unwrap();
    reopened_project.scan_and_import().unwrap();
    let loaded = EditorProjectDocument::load_from_project_for_tests(&reopened_project).unwrap();
    let paths = ProjectPaths::from_root(&root).unwrap();

    assert!(paths.manifest_path().exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("default.zmaterial")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("models")
        .join("cube.obj")
        .exists());

    let reopened_cube = loaded
        .world
        .find_node(cube.id)
        .expect("reopened project must retain the template Cube identity");
    assert_eq!(reopened_cube.id, cube.id);
    assert_eq!(reopened_cube.name, cube.name);
    assert_eq!(reopened_cube.parent, cube.parent);
    assert_eq!(reopened_cube.transform, expected_transform);
    assert_eq!(reopened_cube.mesh, cube.mesh);
    let reopened_mesh = reopened_cube
        .mesh
        .as_ref()
        .expect("the template Cube must retain its mesh renderer");
    let resolved_model = reopened_project
        .asset_registry()
        .resolve_reference_by_asset_id(reopened_mesh.model.id())
        .expect("the reopened registry must resolve the Cube model reference");
    let resolved_material = reopened_project
        .asset_registry()
        .resolve_reference_by_asset_id(reopened_mesh.material.id())
        .expect("the reopened registry must resolve the Cube material reference");
    assert_eq!(
        reopened_project
            .asset_registry()
            .resolve_asset_id_for_reference(resolved_model.uuid, &resolved_model.locator)
            .unwrap(),
        reopened_mesh.model.id()
    );
    assert_eq!(
        reopened_project
            .asset_registry()
            .resolve_asset_id_for_reference(resolved_material.uuid, &resolved_material.locator)
            .unwrap(),
        reopened_mesh.material.id()
    );
    for locator in [&resolved_model.locator, &resolved_material.locator] {
        let record = reopened_project
            .registry()
            .get_by_locator(locator)
            .unwrap_or_else(|| {
                panic!(
                    "the reopened project generation must retain resource {}",
                    locator
                )
            });
        assert_eq!(
            record.state,
            ResourceState::Ready,
            "the reopened project resource {} must import successfully: {}",
            locator,
            record.failure_reason().unwrap_or("no import diagnostic")
        );
        assert!(
            record.artifact_locator().is_some(),
            "the reopened project resource {} must retain an artifact locator",
            locator
        );
    }
    assert_eq!(
        loaded.world.find_node(camera.id),
        Some(camera),
        "saving Cube transform must not alter the Camera"
    );
    assert_eq!(
        loaded.world.find_node(sun.id),
        Some(sun),
        "saving Cube transform must not alter the Sun"
    );
    assert_eq!(
        loaded.manifest.default_scene.to_string(),
        "res://scenes/main.scene.toml"
    );
    assert_eq!(
        loaded.editor_workspace,
        Some(workspace),
        "workspace must roundtrip alongside the persisted scene"
    );

    drop(loaded);
    drop(reopened_project);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn saving_an_explicit_scene_target_never_overwrites_the_manifest_default_scene() {
    let root = unique_mvp_project_root("explicit-scene-save-target");
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let default_scene_uri = project.manifest().default_scene.clone();
    let default_scene_path = project.source_path_for_uri(&default_scene_uri).unwrap();
    let default_before = fs::read(&default_scene_path).unwrap();
    let selected_scene_uri = AssetUri::parse("res://scenes/level_b.scene.toml").unwrap();
    ProjectAuthority::default()
        .create_scene(
            &mut project,
            SceneCreateRequest::new(selected_scene_uri.clone()),
        )
        .unwrap();

    let selected_world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_scene_to_project(
        &project,
        &selected_scene_uri,
        &selected_world,
        None,
    )
    .unwrap();

    assert_eq!(fs::read(default_scene_path).unwrap(), default_before);
    assert!(project
        .source_path_for_uri(&selected_scene_uri)
        .unwrap()
        .is_file());
}

#[test]
fn editor_project_document_current_scene_save_is_byte_stable() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("canonical-save-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let document = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let scene_path = root.join("assets").join("scenes").join("main.scene.toml");

    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &document.world,
        document.editor_workspace.as_ref(),
    )
    .unwrap();
    let first_save = fs::read(&scene_path).unwrap();

    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &document.world,
        document.editor_workspace.as_ref(),
    )
    .unwrap();
    let second_save = fs::read(&scene_path).unwrap();

    assert_eq!(
        first_save, second_save,
        "a current-format project scene must not drift on a second canonical save"
    );

    drop(document);
    drop(project);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn editor_project_document_failed_scene_save_preserves_dirty_baseline_and_last_valid_scene() {
    const TRANSFORM_X_DELTA: f32 = 6.5;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("failed-save-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let document = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let cube = document
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("renderable project must retain the template Cube")
        .clone();
    let level = DefaultLevelManager::default()
        .create_level(document.world.clone(), LevelMetadata::default());
    let mut context = CoreEditContext::default();
    context
        .bind_scene(
            level.clone(),
            SceneSelection::new(vec![cube.id], Some(cube.id)),
        )
        .unwrap();
    let transactions = EditorTransactionEngine::new(context);
    let mut changed_transform = cube.transform.clone();
    changed_transform.translation.x += TRANSFORM_X_DELTA;
    let command = level
        .with_world(|scene| EditorCommand::set_transform(scene, cube.id, changed_transform))
        .unwrap()
        .expect("the fixed F3 transform delta must create an editor command");
    let mut scope = transactions
        .begin(
            "Persist F3 Cube transform through failed save",
            HistoryContextId::Global,
        )
        .unwrap();
    scope.push(command).unwrap();
    scope.commit().unwrap();
    assert!(transactions.is_dirty(HistoryContextId::Global).unwrap());

    let scene_path = root.join("assets").join("scenes").join("main.scene.toml");
    let last_valid_scene = fs::read(&scene_path).unwrap();
    let scene_directory = scene_path
        .parent()
        .expect("default scene must have an assets/scenes parent")
        .to_path_buf();
    let displaced_scene_directory = root.join(format!(".scenes-failed-save-{unique}"));
    fs::rename(&scene_directory, &displaced_scene_directory).unwrap();
    fs::write(&scene_directory, "not a directory").unwrap();

    let error = EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &level.snapshot(),
        document.editor_workspace.as_ref(),
    )
    .unwrap_err();
    assert!(matches!(error, SceneProjectError::Io(_)));
    assert!(transactions.is_dirty(HistoryContextId::Global).unwrap());

    fs::remove_file(&scene_directory).unwrap();
    fs::rename(&displaced_scene_directory, &scene_directory).unwrap();
    assert_eq!(
        fs::read(&scene_path).unwrap(),
        last_valid_scene,
        "a failed scene write must leave the last valid persisted document intact"
    );

    drop(transactions);
    drop(level);
    drop(document);
    drop(project);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn editor_project_document_failed_scene_save_restores_the_previous_workspace() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("workspace-rollback-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let document = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let previous_workspace = ProjectEditorWorkspace {
        workbench: WorkbenchLayout::default(),
        open_view_instances: Vec::new(),
        focused_view: Some(ViewInstanceId::new("scene#before-failed-save")),
        active_drawers: Vec::new(),
    };
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &document.world,
        Some(&previous_workspace),
    )
    .unwrap();
    let changed_workspace = ProjectEditorWorkspace {
        workbench: WorkbenchLayout::default(),
        open_view_instances: Vec::new(),
        focused_view: Some(ViewInstanceId::new("scene#after-failed-save")),
        active_drawers: Vec::new(),
    };

    let scene_path = root.join("assets").join("scenes").join("main.scene.toml");
    let scene_directory = scene_path
        .parent()
        .expect("default scene must have an assets/scenes parent")
        .to_path_buf();
    let displaced_scene_directory = root.join(format!(".scenes-workspace-rollback-{unique}"));
    fs::rename(&scene_directory, &displaced_scene_directory).unwrap();
    fs::write(&scene_directory, "not a directory").unwrap();

    let error = EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &document.world,
        Some(&changed_workspace),
    )
    .unwrap_err();
    assert!(matches!(error, SceneProjectError::Io(_)));

    fs::remove_file(&scene_directory).unwrap();
    fs::rename(&displaced_scene_directory, &scene_directory).unwrap();
    let restored = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    assert_eq!(
        restored.editor_workspace,
        Some(previous_workspace),
        "a failed scene save must restore the previously persisted workspace"
    );

    drop(restored);
    drop(document);
    drop(project);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn editor_project_document_workspace_write_failure_keeps_last_valid_scene() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("workspace-save-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let document = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let mut changed_world = document.world.clone();
    let cube = changed_world
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("renderable project must retain the template Cube")
        .clone();
    let mut changed_transform = cube.transform.clone();
    changed_transform.translation.x += 8.0;
    assert!(changed_world
        .update_transform(cube.id, changed_transform)
        .unwrap());

    let workspace = ProjectEditorWorkspace {
        workbench: WorkbenchLayout::default(),
        open_view_instances: Vec::new(),
        focused_view: None,
        active_drawers: Vec::new(),
    };
    let scene_path = root.join("assets").join("scenes").join("main.scene.toml");
    let last_valid_scene = fs::read(&scene_path).unwrap();
    let workspace_directory = root.join(".zircon");
    let displaced_workspace_directory =
        root.join(format!(".zircon-workspace-failed-save-{unique}"));
    fs::rename(&workspace_directory, &displaced_workspace_directory).unwrap();
    fs::write(&workspace_directory, "not a directory").unwrap();

    let error = EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &changed_world,
        Some(&workspace),
    )
    .unwrap_err();
    assert!(matches!(error, SceneProjectError::Io(_)));

    fs::remove_file(&workspace_directory).unwrap();
    fs::rename(&displaced_workspace_directory, &workspace_directory).unwrap();
    assert_eq!(
        fs::read(&scene_path).unwrap(),
        last_valid_scene,
        "a workspace write failure must not commit the authoring scene"
    );

    drop(document);
    drop(project);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn editor_project_document_projects_persisted_missing_and_invalid_project_settings() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("project-settings-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let settings_path = root.join(".zircon").join("settings.toml");

    let persisted = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    assert_eq!(
        persisted.project_settings,
        ProjectSettingsLoadState::Persisted {
            path: settings_path.clone(),
            schema_version: 1,
        }
    );
    assert_eq!(persisted.project_settings.startup_status(), "persisted-v1");

    fs::remove_file(&settings_path).unwrap();
    let missing = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    assert_eq!(
        missing.project_settings,
        ProjectSettingsLoadState::Missing {
            path: settings_path.clone(),
        }
    );
    assert_eq!(
        missing.project_settings.startup_status(),
        "degraded-missing"
    );

    fs::write(&settings_path, "not a versioned settings envelope").unwrap();
    let invalid = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let ProjectSettingsLoadState::Invalid { path, message } = &invalid.project_settings else {
        panic!("corrupt project settings must remain an explicit degraded startup state");
    };
    assert_eq!(path, &settings_path);
    assert!(!message.trim().is_empty());
    assert_eq!(
        invalid.project_settings.startup_status(),
        "degraded-invalid"
    );

    drop(invalid);
    drop(missing);
    drop(persisted);
    drop(project);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn editor_project_document_ignores_unknown_workspace_format_with_diagnostic() {
    let manager = DefaultLevelManager::default();
    let world = manager.create_default_level().snapshot();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("future-workspace-{unique}"));
    create_renderable_project(&root);
    let workspace = ProjectEditorWorkspace {
        workbench: WorkbenchLayout::default(),
        open_view_instances: Vec::new(),
        focused_view: None,
        active_drawers: Vec::new(),
    };

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &world,
        Some(&workspace),
    )
    .unwrap();
    let workspace_path = root.join(".zircon").join("editor-workspace.json");
    let source = fs::read_to_string(&workspace_path)
        .unwrap()
        .replace("\"schema_version\": 1", "\"schema_version\": 999");
    fs::write(&workspace_path, source).unwrap();

    let loaded = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();

    assert!(loaded.editor_workspace.is_none());
    assert_eq!(loaded.workspace_restore_diagnostics.len(), 1);
    assert!(loaded.workspace_restore_diagnostics[0]
        .message
        .contains("version 999 is newer than supported version 1"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn editor_project_document_loads_from_the_active_generation_without_reopening_manifest() {
    let manager = DefaultLevelManager::default();
    let world = manager.create_default_level().snapshot();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = unique_mvp_project_root(format!("generation-{unique}"));
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &world,
        None,
    )
    .unwrap();
    fs::remove_file(root.join("zircon-project.toml")).unwrap();

    let loaded = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();

    assert_eq!(loaded.manifest.name, project.manifest().name);
    assert_eq!(loaded.world.nodes().len(), world.nodes().len());
    let _ = fs::remove_dir_all(&root);
}

fn unique_mvp_project_root(label: impl AsRef<str>) -> PathBuf {
    let executable = std::env::current_exe().expect("locate the F3 test executable");
    let binary_directory = executable
        .parent()
        .expect("F3 test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve the F3 test binary directory");

    binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(label.as_ref())
}

fn create_renderable_project(root: &Path) {
    let project_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("temporary project root must have a UTF-8 file name");
    let location = root
        .parent()
        .expect("temporary project root must have a parent");
    let created = ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: project_name.to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    assert_eq!(created.root, root);
}
