use super::*;
use crate::core::commands::EditorCommandRegistry;
use crate::core::editing::engine::{
    EditCommandCodecRegistry, EditorTransactionEngine, HistoryContextId, TransactionJournal,
    TransactionJournalReplayer,
};
use crate::core::editing::journal_codecs::register_scene_command_codecs;
use crate::core::editor_message::DocumentId;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::play::WorldDomain;
use serde_json::json;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata};

fn binding() -> SceneScriptBindingAsset {
    serde_json::from_value(json!({
        "package": "absent_package", "module": "main", "enabled": true,
        "update": true, "fixed_update": false, "properties": { "role": "player", "orb_total": 8 }
    }))
    .unwrap()
}

fn scene() -> (Scene, NodeId) {
    let mut scene = Scene::empty();
    scene
        .register_component_type(ComponentTypeDescriptor::new("test.Tag", "test", "Tag"))
        .unwrap();
    let id = scene.spawn_node(NodeKind::Empty).unwrap();
    (scene, id)
}

fn engine(scene: Scene, id: NodeId) -> EditorTransactionEngine {
    let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
    let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    context
        .bind_scene(level, SceneSelection::new(vec![id], Some(id)))
        .unwrap();
    EditorTransactionEngine::new(context)
}

fn read(engine: &EditorTransactionEngine, id: NodeId) -> Option<Vec<SceneScriptBindingAsset>> {
    engine
        .with_context::<CoreEditContext, _>(|core| {
            core.with_scene(|scene| read_script_bindings(scene, id).unwrap())
                .unwrap()
        })
        .unwrap()
        .unwrap()
}

fn capture(
    engine: &EditorTransactionEngine,
    id: NodeId,
    after: Option<Vec<SceneScriptBindingAsset>>,
) -> EditorCommand {
    EditorCommand::SetScriptBindings(
        engine
            .with_context::<CoreEditContext, _>(|core| {
                core.with_scene(|scene| {
                    SetScriptBindingsCommand::capture(scene, id, after)
                        .unwrap()
                        .unwrap()
                })
                .unwrap()
            })
            .unwrap()
            .unwrap(),
    )
}

fn commit(engine: &EditorTransactionEngine, command: EditorCommand) -> TransactionJournal {
    let mut transaction = engine
        .begin("Set script bindings", HistoryContextId::Global)
        .unwrap();
    transaction.push(command).unwrap();
    let id = transaction.commit().unwrap();
    engine
        .journal_transaction(HistoryContextId::Global, id)
        .unwrap()
}

#[test]
fn script_bindings_capture_add_edit_enable_remove_and_exact_undo_redo() {
    let (scene, id) = scene();
    assert!(SetScriptBindingsCommand::capture(&scene, id, Some(vec![]))
        .unwrap()
        .is_none());
    let engine = engine(scene, id);
    commit(&engine, capture(&engine, id, Some(vec![binding()])));
    assert_eq!(read(&engine, id), Some(vec![binding()]));
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert_eq!(read(&engine, id), None);
    assert!(engine.redo(HistoryContextId::Global).unwrap());
    assert_eq!(read(&engine, id), Some(vec![binding()]));

    let mut edited = binding();
    edited.enabled = false;
    edited.properties.insert("orb_total".into(), json!(12));
    commit(&engine, capture(&engine, id, Some(vec![edited.clone()])));
    assert_eq!(read(&engine, id), Some(vec![edited.clone()]));
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert_eq!(read(&engine, id), Some(vec![binding()]));
    assert!(engine.redo(HistoryContextId::Global).unwrap());
    commit(&engine, capture(&engine, id, Some(vec![])));
    assert_eq!(read(&engine, id), None);
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert_eq!(read(&engine, id), Some(vec![edited]));
}

#[test]
fn script_bindings_stale_before_rejects_without_overwriting_concurrent_edits() {
    let (mut scene, id) = scene();
    let mut command = EditorCommand::SetScriptBindings(
        SetScriptBindingsCommand::capture(&scene, id, Some(vec![binding()]))
            .unwrap()
            .unwrap(),
    );
    let mut concurrent = binding();
    concurrent.module = "other".into();
    scene
        .set_dynamic_component(
            id,
            SceneScriptBindingAsset::COMPONENT_TYPE_PATH,
            serde_json::to_value(vec![concurrent.clone()]).unwrap(),
        )
        .unwrap();
    let engine = engine(scene, id);
    engine
        .with_context_mut::<CoreEditContext, _>(|core| {
            let error = command.apply(core).unwrap_err();
            assert_eq!(error.effect, CommandEffect::Unchanged);
        })
        .unwrap()
        .unwrap();
    assert_eq!(read(&engine, id), Some(vec![concurrent]));
    assert_eq!(
        engine.history_status(HistoryContextId::Global).unwrap().len,
        0
    );
}

#[test]
fn script_bindings_journal_replay_uses_typed_values_without_discovery_and_remains_undoable() {
    let (source, id) = scene();
    let (target, target_id) = scene();
    assert_eq!(id, target_id);
    let source = engine(source, id);
    let journal = commit(&source, capture(&source, id, Some(vec![binding()])));
    assert_eq!(
        journal.commands()[0].command_type(),
        SET_SCRIPT_BINDINGS_JOURNAL
    );
    assert_eq!(journal.commands()[0].schema_version(), 1);
    assert_eq!(journal.commands()[0].payload()["before"], json!(null));
    assert!(journal.commands()[0].payload()["after"].is_array());
    let journal = TransactionJournal::decode(&journal.encode().unwrap()).unwrap();
    let mut codecs = EditCommandCodecRegistry::new();
    register_scene_command_codecs(&mut codecs).unwrap();
    let target = engine(target, id);
    let history = HistoryContextId::Document(DocumentId::new(612));
    TransactionJournalReplayer::new(&codecs)
        .replay(&target, history, &journal)
        .unwrap();
    assert_eq!(read(&target, id), Some(vec![binding()]));
    assert!(target.undo(history).unwrap());
    assert_eq!(read(&target, id), None);
    assert!(target.redo(history).unwrap());
    assert_eq!(read(&target, id), Some(vec![binding()]));
}

#[test]
fn script_bindings_capture_and_journal_reject_empty_names_and_noop() {
    let (scene, id) = scene();
    for (package, module) in [("", "main"), ("orb_run", "  ")] {
        let mut invalid = binding();
        invalid.package = package.into();
        invalid.module = module.into();
        assert!(
            SetScriptBindingsCommand::capture(&scene, id, Some(vec![invalid.clone()])).is_err()
        );
        assert!(
            SetScriptBindingsCommand::from_journal(SetScriptBindingsPayload {
                node_id: id,
                before: None,
                after: Some(vec![invalid]),
            })
            .is_err()
        );
    }
    assert!(
        SetScriptBindingsCommand::from_journal(SetScriptBindingsPayload {
            node_id: id,
            before: None,
            after: Some(vec![]),
        })
        .is_err()
    );
    assert!(SetScriptBindingsCommand::capture(&scene, u64::MAX, Some(vec![binding()])).is_err());
}

#[test]
fn script_bindings_undo_preserves_existing_nested_properties() {
    let (mut scene, id) = scene();
    let mut original = binding();
    original
        .properties
        .insert("legacy".into(), json!({"nested": [1, false]}));
    scene
        .set_dynamic_component(
            id,
            SceneScriptBindingAsset::COMPONENT_TYPE_PATH,
            serde_json::to_value(vec![original.clone()]).unwrap(),
        )
        .unwrap();
    let engine = engine(scene, id);
    commit(&engine, capture(&engine, id, None));
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert_eq!(read(&engine, id), Some(vec![original]));
}

#[test]
fn script_binding_operation_uses_document_history_and_rejects_changed_scope() {
    let (scene, id) = scene();
    let engine = engine(scene, id);
    let history = HistoryContextId::Document(DocumentId::new(613));
    let scope = engine
        .with_context::<CoreEditContext, _>(|core| {
            let route = core.capture_world_route(WorldDomain::Edit).unwrap();
            AddComponentScope {
                history,
                gateway: GatewayStamp::capture(route.gateway_identity().unwrap()),
                ui_selection_generation: 1,
                core_selection_generation: core.selection_snapshot().generation(),
                world_generation: core.with_scene(|scene| scene.world_generation()).unwrap(),
                targets: vec![id],
                primary: Some(id),
            }
        })
        .unwrap()
        .unwrap();
    let registry = EditorCommandRegistry::default_workbench();
    for stale in [true, false] {
        let mut request = SetScriptBindingsRequest {
            scope: scope.clone(),
            bindings: SetScriptBindingsPayload {
                node_id: id,
                before: None,
                after: Some(vec![binding()]),
            },
        };
        if stale {
            request.scope.world_generation += 1;
        }
        let invocation = EditorOperationInvocation::parse(SET_SCRIPT_BINDINGS_OPERATION)
            .unwrap()
            .with_arguments(serde_json::to_value(request).unwrap());
        let operation = registry
            .operation_factory(&invocation.operation_id)
            .unwrap()
            .create(&invocation)
            .unwrap();
        let (command, context, merge) = operation.into_parts();
        assert_eq!(context, history);
        let result = engine.execute_operation("Set script bindings", context, None, merge, command);
        if stale {
            assert!(result.is_err());
            assert_eq!(read(&engine, id), None);
        } else {
            result.unwrap();
        }
    }
    assert_eq!(engine.history_status(history).unwrap().len, 1);
    assert!(engine.undo(history).unwrap());
    assert_eq!(read(&engine, id), None);
}
