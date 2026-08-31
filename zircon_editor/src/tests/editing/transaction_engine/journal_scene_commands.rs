use serde_json::json;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, LevelSystem, Scene};
use zircon_runtime_interface::math::{Transform, Vec3};
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::command::{
    BatchTransformJournalPayload, BatchTransformTarget, EditorCommand, NodeEditState,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId, TransactionJournal};
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::EditorRuntimeGatewayHandle;

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";

#[test]
fn committed_scene_commands_produce_versioned_journal_payloads() {
    let mut create_scene = Scene::empty();
    let create_selection = create_scene
        .spawn_node(NodeKind::Camera)
        .expect("test scene spawn should succeed");
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
    let _camera = delete_scene
        .spawn_node(NodeKind::Camera)
        .expect("test scene spawn should succeed");
    let deleted = delete_scene
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
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
    assert!(
        delete_journal.commands()[0]
            .payload()
            .get("records")
            .is_none(),
        "the journal is a replay descriptor, not the move-only inverse delta"
    );

    let mut update_scene = Scene::empty();
    let updated = update_scene
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
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
    let reflected = reflected_scene
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
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

    let mut batch_scene = Scene::empty();
    let first = batch_scene.spawn_node(NodeKind::Cube).unwrap();
    let second = batch_scene.spawn_node(NodeKind::Cube).unwrap();
    let batch_command = batch_transform_command(
        &batch_scene,
        [
            (first, Vec3::new(3.0, 0.0, 0.0)),
            (second, Vec3::new(0.0, 4.0, 0.0)),
        ],
    );
    let batch_journal = commit_journal(
        transaction_engine(batch_scene, first),
        "Move scene selection",
        batch_command,
    );
    assert_payload(
        &batch_journal,
        "zircon.editor.scene.batch_transform",
        &json!({}),
    );
    assert_eq!(
        batch_journal.commands()[0].payload()["targets"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

pub(super) fn batch_transform_command(
    scene: &Scene,
    targets: impl IntoIterator<Item = (zircon_runtime::scene::NodeId, Vec3)>,
) -> EditorCommand {
    let targets = targets
        .into_iter()
        .map(|(node_id, translation)| {
            let before = NodeEditState::capture(scene, node_id).unwrap();
            let mut after = before.clone();
            after.transform = Transform::from_translation(translation);
            BatchTransformTarget::new(node_id, before, after).unwrap()
        })
        .collect();
    EditorCommand::from_journal_batch_transform(BatchTransformJournalPayload { targets }).unwrap()
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
