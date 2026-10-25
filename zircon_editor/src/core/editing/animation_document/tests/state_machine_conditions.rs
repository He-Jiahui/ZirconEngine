use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationParameterValue, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::core::resource::AssetReference;

use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

use super::super::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentKind,
    AnimationDocumentMutation, AnimationDocumentRevision, AnimationEditCommand,
};

fn source() -> AnimationAuthoringAsset {
    let graph = AssetReference::from_locator(
        AssetUri::parse("res://animation/condition.graph.zranim").unwrap(),
    );
    AnimationAuthoringAsset::StateMachine(AnimationStateMachineAsset {
        name: Some("Condition editing".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset::graph_ref("Idle", graph.clone()),
            AnimationStateAsset::graph_ref("Run", graph),
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.2,
            exit_time: None,
            interruption: Default::default(),
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "signal".to_string(),
                operator: AnimationConditionOperatorAsset::Greater,
                value: Some(AnimationParameterValue::Scalar(0.5)),
            }],
        }],
        layers: Vec::new(),
    })
}

fn mutation(operator: &str, value_literal: &str) -> AnimationDocumentMutation {
    AnimationDocumentMutation::SetTransitionCondition {
        from_state: "Idle".to_string(),
        to_state: "Run".to_string(),
        parameter_name: "signal".to_string(),
        operator: operator.to_string(),
        value_literal: value_literal.to_string(),
    }
}

#[test]
fn trigger_conditions_compile_and_roundtrip_without_comparison_operands() {
    for literal in ["", "trigger", "stale comparison text"] {
        let mut asset = source();
        assert!(mutation("triggered", literal).apply(&mut asset).unwrap());
        let condition = &asset.as_state_machine().unwrap().transitions[0].conditions[0];
        assert_eq!(
            condition.operator,
            AnimationConditionOperatorAsset::Triggered
        );
        assert_eq!(condition.value, None);
        assert!(asset.compile().is_successful());

        let bytes = asset.to_bytes().unwrap();
        let restored = AnimationAuthoringAsset::from_bytes(
            AnimationAuthoringDocumentKind::StateMachine,
            &bytes,
        )
        .unwrap();
        assert_eq!(restored.as_state_machine(), asset.as_state_machine());
        assert!(restored.compile().is_successful());
        assert!(!mutation("triggered", "").apply(&mut asset).unwrap());

        for invalid in ["", "NaN", "inf", "1,NaN"] {
            let before = asset.to_bytes().unwrap();
            assert!(!mutation("greater", invalid).apply(&mut asset).unwrap());
            assert_eq!(asset.to_bytes().unwrap(), before);
        }
        assert!(mutation("greater", "0.75").apply(&mut asset).unwrap());
        assert_eq!(
            asset.as_state_machine().unwrap().transitions[0].conditions[0].value,
            Some(AnimationParameterValue::Scalar(0.75))
        );
        assert!(asset.compile().is_successful());
    }
}

fn assert_document(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    revision: u64,
    expected: &AnimationAuthoringAsset,
) {
    engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(id).unwrap();
            let document = handle.read();
            assert_eq!(document.revision().value(), revision);
            assert_eq!(
                document.asset().as_state_machine(),
                expected.as_state_machine()
            );
            let compilation = document.compilation();
            assert_eq!(compilation.current_revision(), document.revision());
            assert!(compilation.current_product().is_successful());
            assert_eq!(compilation.last_good_revision(), Some(document.revision()));
            assert!(compilation.last_good_product().unwrap().is_successful());
            let reopened = AnimationAuthoringAsset::from_bytes(
                document.kind(),
                &document.document_bytes().unwrap(),
            )
            .unwrap();
            assert_eq!(reopened.as_state_machine(), expected.as_state_machine());
        })
        .unwrap()
        .unwrap();
}

#[test]
fn trigger_condition_history_preserves_compilation_and_exact_source() {
    let id = DocumentId::new(9181);
    let comparison = source();
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/condition.state_machine.zranim").unwrap(),
            comparison.clone(),
        ))
        .unwrap();
    let change = mutation("triggered", "");
    let (revision, replacement) = context
        .animation_documents()
        .prepare_mutation(id, &change)
        .unwrap()
        .unwrap();
    let triggered = replacement.clone();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let mut scope = engine.begin(change.label(), history).unwrap();
    scope
        .push(AnimationEditCommand::new(
            change.label(),
            id,
            revision,
            replacement,
        ))
        .unwrap();
    scope.commit().unwrap();
    assert_document(&engine, id, 2, &triggered);

    let before = engine.history_status(history).unwrap();
    assert!(engine
        .with_context::<CoreEditContext, _>(|context| context
            .animation_documents()
            .prepare_mutation(id, &change)
            .unwrap()
            .is_none())
        .unwrap()
        .unwrap());
    assert_eq!(engine.history_status(history).unwrap(), before);

    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 3, &comparison);
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 4, &triggered);
}

fn commit_source(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    revision: AnimationDocumentRevision,
    replacement: AnimationAuthoringAsset,
) {
    let mut scope = engine
        .begin(
            "Replace animation test source",
            HistoryContextId::Document(id),
        )
        .unwrap();
    scope
        .push(AnimationEditCommand::new(
            "Replace animation test source",
            id,
            revision,
            replacement,
        ))
        .unwrap();
    scope.commit().unwrap();
}

fn assert_invalid_trigger(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    revision: u64,
    last_good_revision: u64,
) {
    engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(id).unwrap();
            let document = handle.read();
            let compilation = document.compilation();
            assert_eq!(document.revision().value(), revision);
            assert_eq!(compilation.current_revision(), document.revision());
            assert!(!compilation.current_product().is_successful());
            assert!(compilation
                .current_product()
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-STATE-008"));
            assert_eq!(
                compilation.last_good_revision().unwrap().value(),
                last_good_revision
            );
            assert!(compilation.last_good_product().unwrap().is_successful());

            let reopened = AnimationAuthoringAsset::from_bytes(
                document.kind(),
                &document.document_bytes().unwrap(),
            )
            .unwrap();
            assert_eq!(
                reopened.as_state_machine(),
                document.asset().as_state_machine()
            );
            assert!(reopened
                .compile()
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-STATE-008"));
        })
        .unwrap()
        .unwrap();
}

#[test]
fn legacy_trigger_operand_repair_preserves_last_good_through_undo_and_reopen() {
    let id = DocumentId::new(9182);
    let mut context = CoreEditContext::default();
    let document = AnimationAuthoringDocument::new(
        id,
        AssetUri::parse("res://animation/legacy_condition.state_machine.zranim").unwrap(),
        source(),
    );
    let initial_revision = document.revision();
    context.animation_documents_mut().attach(document).unwrap();
    let engine = EditorTransactionEngine::new(context);

    // A legacy saved document may carry the old writer's illegal comparison operand.
    let AnimationAuthoringAsset::StateMachine(mut legacy) = source() else {
        unreachable!()
    };
    legacy.transitions[0].conditions[0].operator = AnimationConditionOperatorAsset::Triggered;
    commit_source(
        &engine,
        id,
        initial_revision,
        AnimationAuthoringAsset::StateMachine(legacy),
    );
    assert_invalid_trigger(&engine, id, 2, 1);

    let (revision, replacement) = engine
        .with_context::<CoreEditContext, _>(|context| {
            context
                .animation_documents()
                .prepare_mutation(id, &mutation("triggered", "stale comparison text"))
                .unwrap()
                .unwrap()
        })
        .unwrap()
        .unwrap();
    let repaired = replacement.clone();
    commit_source(&engine, id, revision, replacement);
    assert_document(&engine, id, 3, &repaired);

    let history = HistoryContextId::Document(id);
    assert!(engine.undo(history).unwrap());
    assert_invalid_trigger(&engine, id, 4, 3);
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 5, &repaired);
}
