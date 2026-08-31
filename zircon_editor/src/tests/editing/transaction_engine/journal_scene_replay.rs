use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, NodeId, Scene};
use zircon_runtime_interface::math::Vec3;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    EditCommandCodecRegistry, EditorTransactionEngine, HistoryContextId, JournalReplayError,
    TransactionJournal, TransactionJournalReplayer,
};
use crate::core::editing::journal_codecs::register_scene_command_codecs;
use crate::core::editing::selection::SceneSelection;
use crate::core::editor_message::DocumentId;
use crate::core::gateway::EditorRuntimeGatewayHandle;

use super::journal_scene_commands::batch_transform_command;

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";

#[test]
fn scene_create_journal_replays_the_captured_record_into_a_matching_baseline() {
    let (source_scene, source_camera, _) = seeded_scene();
    let (target_scene, target_camera, _) = seeded_scene();
    assert_eq!(source_camera, target_camera, "matching baseline node ids");

    let source = transaction_engine(source_scene, source_camera);
    let target = transaction_engine(target_scene, target_camera);
    let journal = commit_journal(
        &source,
        "Create scene node",
        EditorCommand::create_node(NodeKind::Cube),
    );

    replay_scene_journal(&target, &journal);

    assert!(target
        .with_context::<CoreEditContext, _>(|context| {
            context.with_scene(|scene| scene.nodes().iter().any(|node| node.kind == NodeKind::Cube))
        })
        .unwrap()
        .unwrap()
        .unwrap());
}

#[test]
fn scene_update_journal_replays_the_captured_node_state_into_a_matching_baseline() {
    let (source_scene, source_camera, source_cube) = seeded_scene();
    let (target_scene, target_camera, target_cube) = seeded_scene();
    assert_eq!(source_camera, target_camera, "matching baseline camera id");
    assert_eq!(source_cube, target_cube, "matching baseline cube id");

    let command = EditorCommand::rename_node(&source_scene, source_cube, "Recovered Cube".into())
        .unwrap()
        .unwrap();
    let source = transaction_engine(source_scene, source_cube);
    let target = transaction_engine(target_scene, target_cube);
    let journal = commit_journal(&source, "Rename scene node", command);

    replay_scene_journal(&target, &journal);

    assert_eq!(
        target
            .with_context::<CoreEditContext, _>(|context| {
                context
                    .with_scene(|scene| scene.find_node(target_cube).map(|node| node.name.clone()))
            })
            .unwrap()
            .unwrap()
            .unwrap(),
        "Recovered Cube"
    );
}

#[test]
fn scene_delete_journal_replays_without_serializing_the_runtime_inverse_delta() {
    let (source_scene, source_camera, source_cube) = seeded_scene();
    let (target_scene, target_camera, target_cube) = seeded_scene();
    assert_eq!(source_camera, target_camera, "matching baseline camera id");
    assert_eq!(source_cube, target_cube, "matching baseline cube id");

    let command = EditorCommand::delete_node(&source_scene, source_cube).unwrap();
    let source = transaction_engine(source_scene, source_cube);
    let target = transaction_engine(target_scene, target_cube);
    let journal = commit_journal(&source, "Delete scene node", command);

    replay_scene_journal(&target, &journal);

    assert!(!target
        .with_context::<CoreEditContext, _>(|context| {
            context.with_scene(|scene| scene.contains_entity(target_cube))
        })
        .unwrap()
        .unwrap());
}

#[test]
fn reflected_field_journal_replays_through_the_live_scene_reflection_gateway() {
    let (source_scene, source_camera, source_cube) = seeded_scene();
    let (target_scene, target_camera, target_cube) = seeded_scene();
    assert_eq!(source_camera, target_camera, "matching baseline camera id");
    assert_eq!(source_cube, target_cube, "matching baseline cube id");

    let command = EditorCommand::set_reflected_scene_field(
        &source_scene,
        source_cube,
        NAME_TYPE_PATH,
        "value",
        ReflectedValue::String("Recovered Name".into()),
    )
    .unwrap()
    .unwrap();
    let source = transaction_engine(source_scene, source_cube);
    let target = transaction_engine(target_scene, target_cube);
    let journal = commit_journal(&source, "Set reflected scene field", command);

    replay_scene_journal(&target, &journal);

    assert_eq!(
        target
            .with_context::<CoreEditContext, _>(|context| {
                context
                    .with_scene(|scene| scene.find_node(target_cube).map(|node| node.name.clone()))
            })
            .unwrap()
            .unwrap()
            .unwrap(),
        "Recovered Name"
    );
}

#[test]
fn batch_transform_journal_replays_every_target_as_one_command() {
    let (source_scene, source_camera, source_cube) = seeded_scene();
    let (target_scene, target_camera, target_cube) = seeded_scene();
    assert_eq!(source_camera, target_camera);
    assert_eq!(source_cube, target_cube);

    let command = batch_transform_command(
        &source_scene,
        [
            (source_camera, Vec3::new(1.0, 2.0, 3.0)),
            (source_cube, Vec3::new(4.0, 5.0, 6.0)),
        ],
    );
    let source = transaction_engine(source_scene, source_cube);
    let target = transaction_engine(target_scene, target_cube);
    let journal = commit_journal(&source, "Move scene selection", command);

    replay_scene_journal(&target, &journal);

    target
        .with_context::<CoreEditContext, _>(|context| {
            context.with_scene(|scene| {
                assert_eq!(
                    scene.local_transform(target_camera).unwrap().translation,
                    Vec3::new(1.0, 2.0, 3.0)
                );
                assert_eq!(
                    scene.local_transform(target_cube).unwrap().translation,
                    Vec3::new(4.0, 5.0, 6.0)
                );
            })
        })
        .unwrap()
        .unwrap()
        .unwrap();
}

#[test]
fn invalid_scene_payload_is_rejected_before_the_target_history_or_scene_changes() {
    let (source_scene, _, source_cube) = seeded_scene();
    let command = EditorCommand::rename_node(&source_scene, source_cube, "Recovered Cube".into())
        .unwrap()
        .unwrap();
    let source = transaction_engine(source_scene, source_cube);
    let journal = commit_journal(&source, "Rename scene node", command);
    let mut encoded: serde_json::Value = serde_json::from_slice(
        &journal
            .encode()
            .expect("journal should encode for corruption test"),
    )
    .expect("journal envelope should be valid JSON");
    encoded["$zircon"]["payload"]["commands"][0]["payload"]["after"]["name"] =
        serde_json::json!("   ");
    let invalid_journal = TransactionJournal::decode(
        &serde_json::to_vec(&encoded).expect("corrupted journal should encode"),
    )
    .expect("journal envelope remains structurally valid");

    let (target_scene, _, target_cube) = seeded_scene();
    let target = transaction_engine(target_scene, target_cube);
    let original_name = target
        .with_context::<CoreEditContext, _>(|context| {
            context.with_scene(|scene| scene.find_node(target_cube).map(|node| node.name.clone()))
        })
        .expect("target context should be available")
        .expect("target context should have the expected type")
        .expect("target cube should exist");
    let mut codecs = EditCommandCodecRegistry::new();
    register_scene_command_codecs(&mut codecs).expect("scene codecs should register once");
    let history = HistoryContextId::Document(DocumentId::new(611));

    let error = TransactionJournalReplayer::new(&codecs)
        .replay(&target, history, &invalid_journal)
        .expect_err("invalid replay payload must be rejected");

    assert!(matches!(error, JournalReplayError::Decode(_)));
    assert_eq!(
        target
            .with_context::<CoreEditContext, _>(|context| {
                context
                    .with_scene(|scene| scene.find_node(target_cube).map(|node| node.name.clone()))
            })
            .expect("target context should remain available")
            .expect("target context should have the expected type")
            .expect("target cube should remain present"),
        original_name
    );
    assert_eq!(
        target
            .history_status(history)
            .expect("target history should remain available")
            .len,
        0
    );
}

fn seeded_scene() -> (Scene, NodeId, NodeId) {
    let mut scene = Scene::empty();
    let camera = scene
        .spawn_node(NodeKind::Camera)
        .expect("test scene camera spawn should succeed");
    let cube = scene
        .spawn_node(NodeKind::Cube)
        .expect("test scene cube spawn should succeed");
    (scene, camera, cube)
}

fn transaction_engine(scene: Scene, selected: NodeId) -> EditorTransactionEngine {
    let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
    let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    context
        .bind_scene(level, SceneSelection::new(vec![selected], Some(selected)))
        .expect("test scene binding should succeed");
    EditorTransactionEngine::new(context)
}

fn commit_journal(
    engine: &EditorTransactionEngine,
    label: &str,
    command: EditorCommand,
) -> TransactionJournal {
    let mut scope = engine
        .begin(label, HistoryContextId::Global)
        .expect("source transaction should begin");
    scope.push(command).expect("source command should apply");
    let transaction = scope.commit().expect("source transaction should commit");
    engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .expect("source transaction should produce a journal")
}

fn replay_scene_journal(target: &EditorTransactionEngine, journal: &TransactionJournal) {
    let mut codecs = EditCommandCodecRegistry::new();
    register_scene_command_codecs(&mut codecs).expect("scene codecs should register once");
    TransactionJournalReplayer::new(&codecs)
        .replay(
            target,
            HistoryContextId::Document(DocumentId::new(610)),
            journal,
        )
        .expect("scene journal should replay");
}
