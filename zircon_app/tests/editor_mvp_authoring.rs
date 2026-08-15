#![cfg(feature = "target-editor-host")]

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_app::EditorApplicationComposition;
use zircon_editor::core::editor_event::{
    EditorEvent, EditorEventSource, InspectorFieldChange, MenuAction,
};
use zircon_editor::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use zircon_editor::ui::binding::{
    EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, SelectionCommand,
};
use zircon_editor::ui::workbench::project::EditorProjectDocument;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn f4_project_authoring_survives_full_application_restart() {
    let _environment = config_environment_lock().lock().unwrap();
    let location = unique_mvp_project_directory("zircon_app_f4_authoring");
    fs::create_dir_all(&location).unwrap();
    let _config_path = ConfigPathGuard::set(location.join("editor-config.json"));
    let created = ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: "F4Authoring".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let canonical_project_root = ProjectPaths::resolve_existing_path(&created.root).unwrap();

    let first = EditorApplicationComposition::open_project(&created.root).unwrap();
    let opened_project = first.prepared_project();
    assert!(
        opened_project.project_info.asset_count >= 4,
        "the startup summary must come from the post-import project generation"
    );
    assert_eq!(
        opened_project.project_info.failed_asset_count, 0,
        "the RenderableEmpty starter assets must not hide import failures"
    );
    assert_eq!(
        opened_project.project_info.ready_asset_count, opened_project.project_info.asset_count,
        "the RenderableEmpty starter assets must all reach Ready before F4 authoring begins"
    );
    let cube = opened_project
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("renderable-empty project must contain a Cube")
        .id;
    let (initial_cube, initial_camera, initial_sun) = {
        let world = &opened_project.world;
        (
            world
                .find_node(cube)
                .expect("renderable-empty project must retain the Cube identity")
                .clone(),
            world
                .nodes()
                .iter()
                .find(|node| node.name == "Camera")
                .expect("renderable-empty project must retain its Camera")
                .clone(),
            world
                .nodes()
                .iter()
                .find(|node| node.name == "Sun")
                .expect("renderable-empty project must retain its Sun")
                .clone(),
        )
    };

    let selection_binding = EditorUiBinding::new(
        "Hierarchy",
        "SelectSceneNode",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: cube,
        }),
    );
    let transform_binding = EditorUiBinding::new(
        "Inspector",
        "TransformPositionXCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            [InspectorFieldChange {
                field_id: "transform.translation.x".to_string(),
                value: UiBindingValue::Float(42.0),
            }],
        ),
    );
    let scale_binding = EditorUiBinding::new(
        "Inspector",
        "TransformScaleXCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            [InspectorFieldChange {
                field_id: "transform.scale.x".to_string(),
                value: UiBindingValue::Float(1.25),
            }],
        ),
    );
    let undo_binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "Undo",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.history.undo"),
    );
    let redo_binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "Redo",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.history.redo"),
    );
    let save_binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "SaveProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.project.save"),
    );
    let first = first
        .run_retained_host_automation(&[
            selection_binding,
            transform_binding,
            scale_binding,
            undo_binding,
            redo_binding,
            save_binding,
        ])
        .unwrap();
    let first_snapshot = first.editor_snapshot;
    assert!(first_snapshot.project_open);
    assert_eq!(
        PathBuf::from(&first_snapshot.project_path),
        canonical_project_root
    );
    assert!(first_snapshot.scene_entries.iter().any(
        |entry| entry.entity == cube && first_snapshot.scene_entries.is_selected(entry.entity)
    ));
    assert_eq!(
        first_snapshot
            .inspector
            .as_ref()
            .expect("selected Cube must project an inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        42.0
    );
    assert_eq!(
        first_snapshot
            .inspector
            .as_ref()
            .expect("selected Cube must project an inspector")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        1.25
    );
    assert_eq!(first.records.len(), 6);
    assert!(
        first
            .records
            .iter()
            .zip([
                "Hierarchy/SelectCube:onClick",
                "Inspector/TransformPositionXCommit:onSubmit",
                "Inspector/TransformScaleXCommit:onSubmit",
                "WorkbenchMenuBar/Undo:onClick",
                "WorkbenchMenuBar/Redo:onClick",
                "WorkbenchMenuBar/SaveProject:onClick",
            ])
            .all(|(record, binding_path)| {
                record.source == EditorEventSource::Cli
                    && record.binding_path.as_deref() == Some(binding_path)
            }),
        "retained-host automation must report canonical CLI binding evidence"
    );
    assert!(first.records[1].transaction_id.is_some());
    assert!(first.records[2].transaction_id.is_some());
    assert_eq!(
        first.records[3].event,
        EditorEvent::WorkbenchMenu(MenuAction::Undo)
    );
    assert_eq!(
        first.records[4].event,
        EditorEvent::WorkbenchMenu(MenuAction::Redo)
    );
    let save_record = first.records.last().unwrap();
    assert_eq!(
        save_record.operation_id.as_deref(),
        Some("file.project.save")
    );
    assert_eq!(save_record.transaction_id, None);
    assert!(
        save_record.save_generation.is_some(),
        "the successful F4 save must publish its persisted history generation"
    );
    let reopened = EditorApplicationComposition::open_project(&created.root).unwrap();
    let reopened_cube = reopened
        .prepared_project()
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("reopened project must contain a Cube")
        .id;
    let reopened = reopened
        .run_retained_host_automation(&[EditorUiBinding::new(
            "Hierarchy",
            "SelectSceneNode",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
                node_id: reopened_cube,
            }),
        )])
        .unwrap();
    let reopened_snapshot = reopened.editor_snapshot;
    assert_eq!(
        reopened_snapshot
            .inspector
            .as_ref()
            .expect("reopened Cube must project an inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        42.0
    );
    assert_eq!(
        reopened_snapshot
            .inspector
            .as_ref()
            .expect("reopened Cube must project an inspector")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        1.25
    );
    assert_eq!(
        reopened_cube, cube,
        "reopening must preserve the persisted Cube entity identity"
    );

    let mut opened_project = ProjectAuthority::default()
        .open_project(&created.root)
        .unwrap()
        .into_project();
    opened_project.scan_and_import().unwrap();
    let persisted = EditorProjectDocument::load_from_project(&opened_project).unwrap();
    let persisted_cube = persisted
        .world
        .find_node(reopened_cube)
        .expect("persisted project must retain the Cube");
    let mut expected_transform = initial_cube.transform.clone();
    expected_transform.translation.x = 42.0;
    expected_transform.scale.x = 1.25;
    assert_eq!(persisted_cube.id, initial_cube.id);
    assert_eq!(persisted_cube.name, initial_cube.name);
    assert_eq!(persisted_cube.parent, initial_cube.parent);
    assert_eq!(persisted_cube.transform, expected_transform);
    assert_eq!(
        persisted_cube.mesh, initial_cube.mesh,
        "project-scoped mesh and material references must resolve into the reopened generation"
    );
    assert_eq!(
        persisted.world.find_node(initial_camera.id),
        Some(initial_camera),
        "saving the Cube must not alter the persisted Camera"
    );
    assert_eq!(
        persisted.world.find_node(initial_sun.id),
        Some(initial_sun),
        "saving the Cube must not alter the persisted Sun"
    );

    drop(persisted);
    drop(opened_project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn f4_project_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = unique_mvp_project_directory("physical-root");
    let executable = std::env::current_exe().expect("locate the F4 test executable");
    let binary_directory = executable
        .parent()
        .expect("F4 test executable must have a parent directory");
    let resolved_binary_directory =
        ProjectPaths::resolve_existing(binary_directory).expect("resolve F4 test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "F4 project fixture output must retain the test binary's physical output root"
    );
}

fn unique_mvp_project_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let executable = std::env::current_exe().expect("locate the F4 test executable");
    let binary_directory = executable
        .parent()
        .expect("F4 test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve the F4 test binary directory");

    binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(format!("{prefix}_{}_{}", std::process::id(), nonce))
}

fn config_environment_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct ConfigPathGuard {
    previous: Option<OsString>,
}

impl ConfigPathGuard {
    fn set(path: PathBuf) -> Self {
        let previous = std::env::var_os("ZIRCON_CONFIG_PATH");
        std::env::set_var("ZIRCON_CONFIG_PATH", path);
        Self { previous }
    }
}

impl Drop for ConfigPathGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var("ZIRCON_CONFIG_PATH", previous);
        } else {
            std::env::remove_var("ZIRCON_CONFIG_PATH");
        }
    }
}
