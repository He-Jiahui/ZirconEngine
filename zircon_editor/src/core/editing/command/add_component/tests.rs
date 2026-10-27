use super::*;
use crate::core::commands::EditorCommandRegistry;
use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryRetentionBudgetV1, HistoryRetentionDomainV1,
    OperationTransactionResult,
};
use crate::core::editor_message::DocumentId;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use serde_json::{json, Value};
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::components::{Name, NodeKind};
use zircon_runtime::scene::reflect::VmTypeBacking;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectSerializationStrategy, ReflectTypeInfo,
    ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
};

const TYPE: &str = "weather.Component.CloudLayer";

fn fixture(record_bytes: u64) -> (EditorTransactionEngine, Vec<NodeId>, HistoryContextId) {
    let mut scene = Scene::empty();
    scene
        .register_component_type(
            ComponentTypeDescriptor::new(TYPE, "weather", "Cloud Layer")
                .with_property("coverage", "Scalar", true)
                .with_property("label", "String", false),
        )
        .unwrap();
    fixture_with_scene(scene, record_bytes, 2)
}

fn fixture_with_scene(
    mut scene: Scene,
    record_bytes: u64,
    target_count: usize,
) -> (EditorTransactionEngine, Vec<NodeId>, HistoryContextId) {
    let targets: Vec<_> = (0..target_count)
        .map(|index| scene.spawn((Name(format!("Target {index}")),)).unwrap())
        .collect();
    let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
    let mut core = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    core.bind_scene(
        level,
        SceneSelection::new(targets.clone(), Some(targets[0])),
    )
    .unwrap();
    let domain = HistoryRetentionDomainV1::try_new(HistoryRetentionBudgetV1 {
        record_bytes,
        ..Default::default()
    })
    .unwrap();
    (
        EditorTransactionEngine::with_retention_domain_v1(core, 16, domain).unwrap(),
        targets,
        HistoryContextId::Document(DocumentId::new(7)),
    )
}

fn request(engine: &EditorTransactionEngine, history: HistoryContextId) -> AddComponentRequest {
    let mut request = default_request(engine, history);
    request.component.value = json!({"coverage": 0.25, "label": "thin"});
    request
}

fn default_request(
    engine: &EditorTransactionEngine,
    history: HistoryContextId,
) -> AddComponentRequest {
    engine
        .with_context::<CoreEditContext, _>(|core| {
            let selection = core.scene_selection().unwrap();
            let route = core.capture_world_route(WorldDomain::Edit).unwrap();
            let scope = AddComponentScope {
                history,
                gateway: GatewayStamp::capture(route.gateway_identity().unwrap()),
                ui_selection_generation: 1,
                core_selection_generation: core.selection_snapshot().generation(),
                world_generation: core.with_scene(|scene| scene.world_generation()).unwrap(),
                targets: selection.items().to_vec(),
                primary: selection.primary(),
            };
            let component = core
                .with_scene(|scene| {
                    scene
                        .dynamic_component_creation_template(TYPE)
                        .unwrap()
                        .spec
                })
                .unwrap();
            AddComponentRequest { scope, component }
        })
        .unwrap()
        .unwrap()
}

fn execute(
    engine: &EditorTransactionEngine,
    request: AddComponentRequest,
) -> Result<OperationTransactionResult, EditCommandError> {
    let registry = EditorCommandRegistry::default_workbench();
    let invocation = EditorOperationInvocation::parse(ADD_COMPONENT_OPERATION)
        .unwrap()
        .with_arguments(serde_json::to_value(request).unwrap());
    let operation = registry
        .operation_factory(&invocation.operation_id)
        .unwrap()
        .create(&invocation)
        .unwrap();
    let (command, history, merge) = operation.into_parts();
    engine.execute_operation("Add component", history, None, merge, command)
}

fn values(engine: &EditorTransactionEngine, targets: &[NodeId]) -> Vec<Option<Value>> {
    engine
        .with_context::<CoreEditContext, _>(|core| {
            core.with_scene(|scene| {
                targets
                    .iter()
                    .map(|&target| scene.dynamic_component(target, TYPE).cloned())
                    .collect()
            })
            .unwrap()
        })
        .unwrap()
        .unwrap()
}

fn mutate(engine: &EditorTransactionEngine, write: impl FnOnce(&mut Scene)) {
    engine
        .with_context::<CoreEditContext, _>(|core| {
            let (_, errors) = core.with_scene_mut(write).unwrap().into_parts();
            assert!(errors.is_empty());
        })
        .unwrap()
        .unwrap();
}

#[test]
fn registered_add_operation_commits_one_document_record_and_round_trips_multi_target_undo_redo() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    let request = request(&engine, history);
    let initial_selection = engine
        .with_context::<CoreEditContext, _>(CoreEditContext::scene_selection)
        .unwrap()
        .unwrap()
        .unwrap();
    let expected = request.component.value.clone();
    let execution = execute(&engine, request).unwrap();
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()), Some(expected.clone())]
    );
    assert_eq!(engine.history_status(history).unwrap().len, 1);
    let journal = engine
        .journal_transaction(history, execution.transaction_id)
        .unwrap();
    assert_eq!(journal.commands().len(), 1);
    let payload = &journal.commands()[0];
    assert_eq!(payload.command_type(), JOURNAL_TYPE);
    assert_eq!(payload.schema_version(), 1);
    assert_eq!(payload.payload()["targets"], json!(targets));
    assert_eq!(payload.payload()["component"]["type_path"], TYPE);
    assert_eq!(payload.payload()["component"]["value"], expected);
    assert!(payload.payload()["component"]["schema_json"]
        .as_str()
        .unwrap()
        .contains("coverage"));
    assert!(engine.undo(history).unwrap());
    assert_eq!(values(&engine, &targets), vec![None, None]);
    assert!(engine.redo(history).unwrap());
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()), Some(expected)]
    );
    let selection = engine
        .with_context::<CoreEditContext, _>(CoreEditContext::scene_selection)
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(selection.items(), initial_selection.items());
    assert_eq!(selection.primary(), initial_selection.primary());
}

#[test]
fn add_large_declared_default_round_trips_multi_target_undo_redo() {
    let label = "x".repeat(256 * 1024);
    let registration = ReflectTypeRegistration::new(
        ReflectTypePath::new(TYPE, "CloudLayer").unwrap(),
        "Cloud Layer",
        ReflectTypeInfo::json_with_fields(vec![ReflectFieldInfo::from_stable_keys(
            TYPE,
            "label",
            "label",
            "String",
            ReflectEditorHint::String,
        )
        .with_default_value(ReflectedValue::String(label.clone()))]),
        ReflectSerializationStrategy::Json,
    )
    .as_component()
    .with_plugin_id("weather")
    .unwrap();
    let mut scene = Scene::empty();
    scene
        .register_vm_type(registration, VmTypeBacking::DynamicComponent)
        .unwrap();
    assert!(scene
        .dynamic_component_creation_template(TYPE)
        .unwrap()
        .required_fields
        .is_empty());
    // Staging clones the default for each target; history owns one payload and
    // its schema. Keep the ordinary budgets while making staging exceed that owner.
    let (engine, targets, history) =
        fixture_with_scene(scene, HistoryRetentionBudgetV1::default().record_bytes, 4);
    let request = default_request(&engine, history);
    let expected = json!({"label": label});
    assert_eq!(request.component.value, expected);
    assert_eq!(values(&engine, &targets), vec![None; targets.len()]);

    execute(&engine, request).expect("large declared defaults must be admitted before mutation");
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()); targets.len()]
    );
    assert_eq!(engine.history_status(history).unwrap().len, 1);
    assert!(engine
        .undo(history)
        .expect("staged removal must use the materialization allowance"));
    assert_eq!(values(&engine, &targets), vec![None; targets.len()]);
    assert!(engine.history_status(history).unwrap().can_redo);
    assert!(engine
        .redo(history)
        .expect("staged creation must use the materialization allowance"));
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected); targets.len()]
    );
    assert_eq!(engine.history_status(history).unwrap().len, 1);
}

#[test]
fn add_rejects_missing_wrong_and_extra_initial_fields_before_world_or_history_changes() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    for value in [
        json!({}),
        json!({"coverage":0.25}),
        json!({"coverage":"bad","label":"thin"}),
        json!({"coverage":0.25,"label":"thin","unknown":true}),
    ] {
        let mut candidate = request(&engine, history);
        candidate.component.value = value;
        assert!(execute(&engine, candidate).is_err());
        assert_eq!(values(&engine, &targets), vec![None, None]);
        assert_eq!(engine.history_status(history).unwrap().len, 0);
    }
}

#[test]
fn add_rejects_duplicate_type_without_overwriting_or_partially_adding_other_targets() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    let foreign = json!({"coverage":0.9,"label":"foreign"});
    mutate(&engine, |scene| {
        scene
            .set_dynamic_component(targets[1], TYPE, foreign.clone())
            .unwrap();
    });
    assert!(execute(&engine, request(&engine, history)).is_err());
    assert_eq!(values(&engine, &targets), vec![None, Some(foreign)]);
    assert_eq!(engine.history_status(history).unwrap().len, 0);
}

#[test]
fn add_rejects_stale_selection_schema_and_world_preparations() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    let stale_selection = request(&engine, history);
    engine
        .with_context_mut::<CoreEditContext, _>(|core| {
            core.set_scene_selection(SceneSelection::new(targets.clone(), Some(targets[0])))
                .unwrap();
        })
        .unwrap()
        .unwrap();
    assert!(execute(&engine, stale_selection).is_err());
    let mut changed_schema = request(&engine, history);
    changed_schema.component.schema_json.push(' ');
    assert!(execute(&engine, changed_schema).is_err());
    let stale_world = request(&engine, history);
    mutate(&engine, |scene| {
        scene.spawn((Name("Unrelated".into()),)).unwrap();
    });
    assert!(execute(&engine, stale_world).is_err());
    assert_eq!(values(&engine, &targets), vec![None, None]);
    assert_eq!(engine.history_status(history).unwrap().len, 0);
}

#[test]
fn add_undo_rejects_same_type_same_json_foreign_recreation_for_the_whole_cohort() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    let request = request(&engine, history);
    let expected = request.component.value.clone();
    execute(&engine, request).unwrap();
    let before = engine.history_status(history).unwrap();
    mutate(&engine, |scene| {
        scene.remove_dynamic_component(targets[1], TYPE).unwrap();
        scene
            .set_dynamic_component(targets[1], TYPE, expected.clone())
            .unwrap();
    });
    assert!(engine.undo(history).is_err());
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()), Some(expected)]
    );
    let after = engine.history_status(history).unwrap();
    assert_eq!(after.top, before.top);
    assert_eq!(after.len, before.len);
    assert!(!after.can_redo);
}

#[test]
fn add_history_budget_refuses_owned_json_and_creation_receipts_before_mutation() {
    let (engine, targets, history) = fixture(1024);
    let mut request = request(&engine, history);
    request.component.value["label"] = Value::String("x".repeat(8192));
    assert!(execute(&engine, request).is_err());
    assert_eq!(values(&engine, &targets), vec![None, None]);
    assert_eq!(engine.history_status(history).unwrap().len, 0);
    assert!(!engine.undo(history).unwrap());
}

#[test]
fn add_undo_remains_valid_after_an_unrelated_ordinary_command_is_undone() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    let request = request(&engine, history);
    let expected = request.component.value.clone();
    execute(&engine, request).unwrap();
    let mut scope = engine.begin("Unrelated node", history).unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Mesh))
        .unwrap();
    scope.commit().unwrap();
    assert_eq!(engine.history_status(history).unwrap().len, 2);
    assert!(engine.undo(history).unwrap());
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()), Some(expected)]
    );
    assert!(engine.undo(history).unwrap());
    assert_eq!(values(&engine, &targets), vec![None, None]);
    assert!(engine.redo(history).unwrap());
}

#[test]
fn add_journal_replays_through_registered_codec_with_fresh_instance_receipts_and_document_undo() {
    let (source, targets, history) = fixture(64 * 1024 * 1024);
    let request = request(&source, history);
    let expected = request.component.value.clone();
    let execution = execute(&source, request).unwrap();
    let journal = source
        .journal_transaction(history, execution.transaction_id)
        .unwrap();
    let (target, target_ids, _) = fixture(64 * 1024 * 1024);
    assert_eq!(
        target_ids, targets,
        "the restored document must preserve the journal's entity ids"
    );
    let target_history = HistoryContextId::Document(DocumentId::new(99));
    let mut codecs = crate::core::editing::engine::EditCommandCodecRegistry::default();
    crate::core::editing::journal_codecs::register_scene_command_codecs(&mut codecs).unwrap();
    crate::core::editing::engine::TransactionJournalReplayer::new(&codecs)
        .replay(&target, target_history, &journal)
        .unwrap();
    assert_eq!(
        values(&target, &targets),
        vec![Some(expected.clone()), Some(expected.clone())]
    );
    assert_eq!(target.history_status(target_history).unwrap().len, 1);
    assert_eq!(target.history_status(history).unwrap().len, 0);
    assert!(target.undo(target_history).unwrap());
    assert_eq!(values(&target, &targets), vec![None, None]);
    assert!(target.redo(target_history).unwrap());
    assert_eq!(
        values(&target, &targets),
        vec![Some(expected.clone()), Some(expected)]
    );
}

#[test]
fn add_then_ordinary_delete_undo_restores_the_same_instances_before_add_undo() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    // Ordinary deletion requires a surviving camera under the B5 policy.
    mutate(&engine, |scene| {
        scene.spawn_node(NodeKind::Camera).unwrap();
    });
    let request = request(&engine, history);
    let expected = request.component.value.clone();
    execute(&engine, request).unwrap();
    let delete = engine
        .with_context::<CoreEditContext, _>(|core| {
            core.with_scene(|scene| EditorCommand::delete_node(scene, targets[0]))
                .unwrap()
        })
        .unwrap()
        .unwrap()
        .unwrap();
    let mut scope = engine.begin("Delete component owner", history).unwrap();
    scope.push(delete).unwrap();
    scope.commit().unwrap();
    assert_eq!(
        values(&engine, &targets),
        vec![None, Some(expected.clone())]
    );
    assert!(engine.undo(history).unwrap());
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()), Some(expected)]
    );
    assert!(engine.undo(history).unwrap());
    assert_eq!(values(&engine, &targets), vec![None, None]);
}

#[test]
fn add_history_rejects_a_replacement_world_without_removing_its_equal_json_components() {
    let (engine, targets, history) = fixture(64 * 1024 * 1024);
    let request = request(&engine, history);
    let expected = request.component.value.clone();
    execute(&engine, request).unwrap();
    engine
        .with_context_mut::<CoreEditContext, _>(|core| {
            let replacement = core.with_scene(Clone::clone).unwrap();
            let selection = core.scene_selection().unwrap();
            let level =
                DefaultLevelManager::default().create_level(replacement, LevelMetadata::default());
            core.bind_scene(level, selection).unwrap();
        })
        .unwrap()
        .unwrap();
    assert!(engine.undo(history).is_err());
    assert_eq!(
        values(&engine, &targets),
        vec![Some(expected.clone()), Some(expected)]
    );
    assert_eq!(engine.history_status(history).unwrap().len, 1);
}
