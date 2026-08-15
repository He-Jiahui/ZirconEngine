use super::super::*;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_event::SelectionHostEvent;
use crate::core::editor_message::{EditorMessagePayload, EditorTopic, TOPIC_SCENE_INSPECTION};
use crate::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use crate::ui::workbench::project::EditorProjectDocument;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::components::NodeKind;

#[test]
fn open_project_menu_event_requests_welcome_surface_without_project_open_side_effects() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_open_project");
    let record = runtime
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::OpenProject),
            EditorEventSource::Headless,
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentWelcomeRequested));
    assert!(!record
        .effects
        .contains(&EditorEventEffect::ProjectOpenRequested));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Open an existing project or create a renderable empty project."
    );
}

#[test]
fn replacing_the_editor_world_publishes_an_inspection_resync() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_world_replacement_inspection");
    let topic = EditorTopic::parse(TOPIC_SCENE_INSPECTION).expect("valid scene inspection topic");
    let subscriber = runtime
        .runtime
        .context()
        .bus()
        .register_subscriber([topic])
        .expect("register scene inspection subscriber");
    let replacement = zircon_runtime::scene::create_default_level(&runtime.core.handle())
        .expect("replacement level should build");
    let replacement_generation = replacement.snapshot().world_generation();

    runtime
        .runtime
        .replace_world(replacement, "replacement-project")
        .expect("runtime should adopt the replacement level");

    let deliveries = runtime.runtime.context().bus().drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    let EditorMessagePayload::SceneInspection(message) = deliveries[0].message().payload() else {
        panic!("world replacement must publish a typed scene inspection message");
    };
    assert_eq!(message.previous_generation(), None);
    assert!(message.requires_resync());
    assert_eq!(message.generation(), replacement_generation);
}

#[test]
fn save_project_marks_the_transaction_history_only_after_persisting_the_world() {
    let _guard = env_lock().lock().unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon_editor_save_history_{unique}"));
    let location = root
        .parent()
        .expect("temporary project root should have a parent");
    ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: root
                .file_name()
                .expect("temporary project root should have a name")
                .to_string_lossy()
                .into_owned(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .expect("renderable template project should be created");

    {
        let runtime = EventRuntimeHarness::new("zircon_editor_event_save_history");
        let manager = runtime
            .core
            .resolve_manager::<crate::ui::host::EditorManager>(
                crate::ui::host::module::EDITOR_MANAGER_NAME,
            )
            .expect("editor manager should resolve");
        let document = manager.open_project(&root).expect("project should open");
        let level = manager
            .create_runtime_level(document.world)
            .expect("opened project scene should create a runtime level");
        runtime
            .runtime
            .replace_world(level, root.to_string_lossy())
            .expect("runtime should adopt the opened project level");

        let cube = {
            let shell = runtime.runtime.shell().lock();
            shell
                .state
                .world
                .try_with_world(|scene| {
                    scene
                        .nodes()
                        .iter()
                        .find(|node| node.kind == NodeKind::Cube)
                        .map(|node| node.id)
                })
                .flatten()
                .expect("renderable template should contain a cube")
        };
        runtime
            .runtime
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Selection(SelectionHostEvent::SelectSceneNode { node_id: cube }),
            )
            .expect("hierarchy selection should dispatch");
        runtime
            .runtime
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Inspector(EditorInspectorEvent {
                    subject_path: "entity://selected".to_string(),
                    changes: vec![InspectorFieldChange::new(
                        "transform.translation.x",
                        UiBindingValue::string("4.25"),
                    )],
                }),
            )
            .expect("inspector transaction should dispatch");
        assert!(runtime
            .runtime
            .context()
            .transactions()
            .is_dirty(HistoryContextId::Global)
            .expect("transaction dirty state should be queryable"));

        let save_binding = menu_action_binding(&MenuAction::SaveProject);
        let save_binding_path = save_binding.path().native_prefix();
        let save_record = runtime
            .runtime
            .dispatch_binding(save_binding, EditorEventSource::RetainedHost)
            .expect("save project menu binding should dispatch");
        assert_eq!(
            save_record.binding_path.as_deref(),
            Some(save_binding_path.as_str())
        );
        assert_eq!(
            save_record.operation_id.as_deref(),
            Some("file.project.save")
        );
        assert_eq!(save_record.transaction_id, None);
        assert!(save_record.save_generation.is_some());
        assert!(!runtime
            .runtime
            .context()
            .transactions()
            .is_dirty(HistoryContextId::Global)
            .expect("successful save should mark the current history clean"));
    }

    let mut reopened = ProjectManager::open(&root).expect("saved project should reopen");
    reopened
        .scan_and_import()
        .expect("reopened project assets should scan");
    let document = EditorProjectDocument::load_from_project_for_tests(&reopened)
        .expect("reopened project document should load");
    let cube_x = document
        .world
        .nodes()
        .iter()
        .find(|node| node.kind == NodeKind::Cube)
        .expect("reopened template should retain the cube")
        .transform
        .translation
        .x;
    assert_eq!(cube_x, 4.25);

    let _ = fs::remove_dir_all(root);
}
