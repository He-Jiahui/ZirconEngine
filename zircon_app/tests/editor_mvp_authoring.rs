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
    let location = unique_temp_dir("zircon_app_f4_authoring");
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
    let host = first.editor_host();
    let first_snapshot = host.editor_snapshot();
    assert!(first_snapshot.project_open);
    assert_eq!(
        PathBuf::from(&first_snapshot.project_path),
        canonical_project_root
    );
    let opened_project = first
        .startup_session()
        .project
        .as_ref()
        .expect("project startup must retain the activated project summary");
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
    assert!(
        first
            .startup_session()
            .status_message
            .starts_with("Project opened:"),
        "F4 authoring must not proceed from a degraded project-open state: {}",
        first.startup_session().status_message
    );
    assert!(
        first.startup_session().status_message.contains(&format!(
            "assets={} ready={} failed={} registry_diagnostics={}",
            opened_project.project_info.asset_count,
            opened_project.project_info.ready_asset_count,
            opened_project.project_info.failed_asset_count,
            opened_project.project_info.registry_diagnostic_count,
        )),
        "the user-visible startup diagnostic must describe the activated generation"
    );
    assert!(
        first
            .startup_session()
            .status_message
            .contains("project_settings=persisted-v1"),
        "the user-visible startup diagnostic must identify the persisted project settings source"
    );
    let cube = first_snapshot
        .scene_entries
        .iter()
        .find(|entry| entry.display_name == "Cube")
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
        "SelectCube",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: cube,
        }),
    );
    let selection_binding_path = selection_binding.path().native_prefix();
    let selection_record = host
        .dispatch_binding(selection_binding, EditorEventSource::Headless)
        .unwrap();
    assert_eq!(
        selection_record.binding_path.as_deref(),
        Some(selection_binding_path.as_str()),
        "the F4 selection must originate from the normal Hierarchy binding"
    );
    let selected_snapshot = host.editor_snapshot();
    assert!(
        selected_snapshot
            .scene_entries
            .iter()
            .any(|entry| entry.entity == cube
                && selected_snapshot.scene_entries.is_selected(entry.entity))
    );
    let initial_x = host
        .editor_snapshot()
        .inspector
        .expect("selected Cube must project an inspector")
        .translation[0]
        .parse::<f32>()
        .unwrap();
    let initial_scale_x = host
        .editor_snapshot()
        .inspector
        .expect("selected Cube must project an inspector")
        .scale[0]
        .parse::<f32>()
        .unwrap();

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
    let transform_binding_path = transform_binding.path().native_prefix();
    let transform_record = host
        .dispatch_binding(transform_binding, EditorEventSource::Headless)
        .unwrap();
    assert_eq!(
        transform_record.binding_path.as_deref(),
        Some(transform_binding_path.as_str()),
        "the F4 transform edit must retain its Inspector binding provenance"
    );
    assert_eq!(
        transform_record.operation_id.as_deref(),
        Some("inspector.field.apply_batch")
    );
    assert!(
        transform_record.transaction_id.is_some(),
        "the committed transform edit must create a transaction"
    );
    assert_eq!(transform_record.save_generation, None);
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("selected Cube must project an inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        42.0
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
    let scale_binding_path = scale_binding.path().native_prefix();
    let scale_record = host
        .dispatch_binding(scale_binding, EditorEventSource::Headless)
        .unwrap();
    assert_eq!(
        scale_record.binding_path.as_deref(),
        Some(scale_binding_path.as_str()),
        "the F4 scale edit must retain its Inspector binding provenance"
    );
    assert_eq!(
        scale_record.operation_id.as_deref(),
        Some("inspector.field.apply_batch")
    );
    assert!(
        scale_record.transaction_id.is_some(),
        "the committed scale edit must create a transaction"
    );
    assert_eq!(scale_record.save_generation, None);
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("selected Cube must project an inspector")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        1.25
    );

    host.dispatch_event(
        EditorEventSource::Headless,
        EditorEvent::WorkbenchMenu(MenuAction::Undo),
    )
    .unwrap();
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("undo must retain the selected Cube inspector")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        initial_scale_x
    );
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("undo must retain the selected Cube inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        42.0
    );

    host.dispatch_event(
        EditorEventSource::Headless,
        EditorEvent::WorkbenchMenu(MenuAction::Undo),
    )
    .unwrap();
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("second undo must retain the selected Cube inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        initial_x
    );
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("second undo must retain the selected Cube inspector")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        initial_scale_x
    );

    host.dispatch_event(
        EditorEventSource::Headless,
        EditorEvent::WorkbenchMenu(MenuAction::Redo),
    )
    .unwrap();
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("redo must restore the selected Cube inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        42.0
    );
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("redo must retain the selected Cube scale before its transaction is replayed")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        initial_scale_x
    );

    host.dispatch_event(
        EditorEventSource::Headless,
        EditorEvent::WorkbenchMenu(MenuAction::Redo),
    )
    .unwrap();
    assert_eq!(
        host.editor_snapshot()
            .inspector
            .expect("second redo must restore the selected Cube inspector")
            .scale[0]
            .parse::<f32>()
            .unwrap(),
        1.25
    );

    let save_binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "SaveProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.project.save"),
    );
    let save_binding_path = save_binding.path().native_prefix();
    let save_record = host
        .dispatch_binding(save_binding, EditorEventSource::Headless)
        .unwrap();
    assert_eq!(
        save_record.binding_path.as_deref(),
        Some(save_binding_path.as_str()),
        "the F4 save must use the normal menu binding instead of a direct event shortcut"
    );
    assert_eq!(
        save_record.operation_id.as_deref(),
        Some("file.project.save")
    );
    assert_eq!(save_record.transaction_id, None);
    assert!(
        save_record.save_generation.is_some(),
        "the successful F4 save must publish its persisted history generation"
    );
    first.close().unwrap();

    let reopened = EditorApplicationComposition::open_project(&created.root).unwrap();
    let reopened_snapshot = reopened.editor_host().editor_snapshot();
    let reopened_cube = reopened_snapshot
        .scene_entries
        .iter()
        .find(|entry| entry.display_name == "Cube")
        .expect("reopened project must contain a Cube")
        .id;
    reopened
        .editor_host()
        .dispatch_binding(
            EditorUiBinding::new(
                "Hierarchy",
                "SelectCube",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
                    node_id: reopened_cube,
                }),
            ),
            EditorEventSource::Headless,
        )
        .unwrap();
    assert_eq!(
        reopened
            .editor_host()
            .editor_snapshot()
            .inspector
            .expect("reopened Cube must project an inspector")
            .translation[0]
            .parse::<f32>()
            .unwrap(),
        42.0
    );
    assert_eq!(
        reopened
            .editor_host()
            .editor_snapshot()
            .inspector
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
    reopened.close().unwrap();
    fs::remove_dir_all(location).unwrap();
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nonce))
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
