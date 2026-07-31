use serde_json::json;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, LevelSystem, Scene};
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId, TransactionJournal};
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::EditorRuntimeGatewayHandle;

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";

#[test]
fn committed_scene_commands_produce_versioned_journal_payloads() {
    let mut create_scene = Scene::empty();
    let create_selection = create_scene.spawn_node(NodeKind::Camera);
    let create_journal = commit_journal(
        transaction_engine(create_scene, create_selection),
        "Create scene node",
        EditorCommand::create_node(NodeKind::Cube),
    );
    assert_payload(
        &create_journal,
        "zircon.editor.scene.create_node",
        &json!({ "intent": { "Node": { "kind": "Cube" } } }),
    );
    assert_eq!(
        create_journal.commands()[0].payload()["record"]["kind"],
        "Cube"
    );

    let mut delete_scene = Scene::empty();
    let _camera = delete_scene.spawn_node(NodeKind::Camera);
    let deleted = delete_scene.spawn_node(NodeKind::Cube);
    let delete_command = EditorCommand::delete_node(&delete_scene, deleted).unwrap();
    let delete_journal = commit_journal(
        transaction_engine(delete_scene, deleted),
        "Delete scene node",
        delete_command,
    );
    assert_payload(
        &delete_journal,
        "zircon.editor.scene.delete_node",
        &json!({ "root_id": deleted }),
    );
    assert_eq!(
        delete_journal.commands()[0].payload()["records"][0]["kind"],
        "Cube"
    );

    let mut update_scene = Scene::empty();
    let updated = update_scene.spawn_node(NodeKind::Cube);
    let update_command = EditorCommand::rename_node(&update_scene, updated, "Journal Cube".into())
        .unwrap()
        .unwrap();
    let update_journal = commit_journal(
        transaction_engine(update_scene, updated),
        "Rename scene node",
        update_command,
    );
    assert_payload(
        &update_journal,
        "zircon.editor.scene.update_node",
        &json!({ "node_id": updated }),
    );
    assert_eq!(
        update_journal.commands()[0].payload()["after"]["name"],
        "Journal Cube"
    );

    let mut reflected_scene = Scene::empty();
    let reflected = reflected_scene.spawn_node(NodeKind::Cube);
    let reflected_command = EditorCommand::set_reflected_scene_field(
        &reflected_scene,
        reflected,
        NAME_TYPE_PATH,
        "value",
        ReflectedValue::String("Journal Name".into()),
    )
    .unwrap()
    .unwrap();
    let reflected_journal = commit_journal(
        transaction_engine(reflected_scene, reflected),
        "Set reflected scene field",
        reflected_command,
    );
    assert_payload(
        &reflected_journal,
        "zircon.editor.scene.set_reflected_field",
        &json!({
            "node_id": reflected,
            "component_type_path": NAME_TYPE_PATH,
            "field_name": "value",
        }),
    );
    assert_eq!(
        reflected_journal.commands()[0].payload()["after"],
        json!({ "kind": "String", "value": "Journal Name" })
    );
}

fn transaction_engine(
    scene: Scene,
    selected: zircon_runtime::scene::NodeId,
) -> EditorTransactionEngine {
    let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
    let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    context
        .bind_scene(level, SceneSelection::new(vec![selected], Some(selected)))
        .unwrap();
    EditorTransactionEngine::new(context)
}

fn commit_journal(
    engine: EditorTransactionEngine,
    label: &str,
    command: EditorCommand,
) -> TransactionJournal {
    let mut scope = engine.begin(label, HistoryContextId::Global).unwrap();
    scope.push(command).unwrap();
    let transaction = scope.commit().unwrap();
    engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap()
}

fn assert_payload(journal: &TransactionJournal, command_type: &str, expected: &serde_json::Value) {
    let payload = &journal.commands()[0];
    assert_eq!(payload.command_type(), command_type);
    assert_eq!(payload.schema_version(), 1);
    for (key, value) in expected.as_object().unwrap() {
        assert_eq!(
            payload.payload().get(key),
            Some(value),
            "payload field `{key}`"
        );
    }
}
