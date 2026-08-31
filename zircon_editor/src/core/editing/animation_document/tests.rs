use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationParameterValue,
};
use zircon_runtime::core::resource::{AssetReference, ResourceLocator};

use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

use super::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationDocumentRevision, AnimationEditCommand,
};

fn graph_document(id: DocumentId) -> AnimationAuthoringDocument {
    AnimationAuthoringDocument::new(
        id,
        AssetUri::parse("res://animation/hero.graph.zranim")
            .expect("test graph locator must be canonical"),
        AnimationAuthoringAsset::Graph(AnimationGraphAsset {
            name: Some("Hero Graph".to_string()),
            parameters: vec![AnimationGraphParameterAsset {
                name: "grounded".to_string(),
                default_value: AnimationParameterValue::Bool(true),
            }],
            nodes: Vec::new(),
        }),
    )
}

fn valid_graph_document(id: DocumentId) -> AnimationAuthoringDocument {
    AnimationAuthoringDocument::new(
        id,
        AssetUri::parse("res://animation/hero.graph.zranim")
            .expect("test graph locator must be canonical"),
        AnimationAuthoringAsset::Graph(AnimationGraphAsset {
            name: Some("Hero Graph".to_string()),
            parameters: Vec::new(),
            nodes: vec![
                AnimationGraphNodeAsset::Clip {
                    id: "idle".to_string(),
                    clip: AssetReference::from_locator(
                        ResourceLocator::parse("res://animation/hero_idle.clip")
                            .expect("test clip locator must be canonical"),
                    ),
                    playback_speed: 1.0,
                    looping: true,
                },
                AnimationGraphNodeAsset::Output {
                    source: "idle".to_string(),
                },
            ],
        }),
    )
}

#[test]
fn animation_edit_command_swaps_core_source_through_document_history() {
    let document = DocumentId::new(41);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(graph_document(document))
        .expect("document must attach once");
    let mutation = AnimationDocumentMutation::SetGraphParameter {
        parameter_name: "grounded".to_string(),
        value_literal: "false".to_string(),
    };
    let (expected_revision, replacement) = context
        .animation_documents()
        .prepare_mutation(document, &mutation)
        .expect("mutation must prepare")
        .expect("parameter replacement must change source");
    let engine = EditorTransactionEngine::new(context);
    let mut transaction = engine
        .begin(mutation.label(), HistoryContextId::Document(document))
        .expect("document transaction must begin");
    transaction
        .push(AnimationEditCommand::new(
            mutation.label(),
            document,
            expected_revision,
            replacement,
        ))
        .expect("command must apply");
    transaction.commit().expect("transaction must commit");

    let after_apply = engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context
                .animation_documents()
                .handle(document)
                .expect("document must remain attached");
            let document = handle.read();
            let asset = document
                .asset()
                .as_graph()
                .expect("source must stay a graph");
            (
                document.revision().value(),
                asset.parameters[0].default_value.clone(),
            )
        })
        .expect("context must inspect")
        .expect("context type must match");
    assert_eq!(after_apply.0, 2);
    assert_eq!(after_apply.1, AnimationParameterValue::Bool(false));
    assert!(engine
        .is_dirty(HistoryContextId::Document(document))
        .expect("history dirty query must succeed"));

    assert!(engine
        .undo(HistoryContextId::Document(document))
        .expect("undo must succeed"));
    let after_undo = engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(document).unwrap();
            let document = handle.read();
            let asset = document.asset().as_graph().unwrap();
            (
                document.revision().value(),
                asset.parameters[0].default_value.clone(),
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(after_undo.0, 3);
    assert_eq!(after_undo.1, AnimationParameterValue::Bool(true));

    assert!(engine
        .redo(HistoryContextId::Document(document))
        .expect("redo must succeed"));
    let after_redo = engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(document).unwrap();
            let document = handle.read();
            let asset = document.asset().as_graph().unwrap();
            (
                document.revision().value(),
                asset.parameters[0].default_value.clone(),
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(after_redo.0, 4);
    assert_eq!(after_redo.1, AnimationParameterValue::Bool(false));
}

#[test]
fn no_op_mutation_does_not_yield_a_history_replacement() {
    let document = DocumentId::new(42);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(graph_document(document))
        .expect("document must attach once");

    let prepared = context
        .animation_documents()
        .prepare_mutation(
            document,
            &AnimationDocumentMutation::SetGraphParameter {
                parameter_name: "grounded".to_string(),
                value_literal: "true".to_string(),
            },
        )
        .expect("no-op mutation preparation must not fail");

    assert!(prepared.is_none());
}

#[test]
fn removing_a_missing_graph_node_records_dangling_reference_cleanup() {
    let document = DocumentId::new(43);
    let mut context = CoreEditContext::default();
    let mut graph = graph_document(document);
    let AnimationAuthoringAsset::Graph(asset) = &mut graph.asset else {
        panic!("graph fixture must retain graph source");
    };
    asset.nodes.push(AnimationGraphNodeAsset::Output {
        source: "stale_node".to_string(),
    });
    context
        .animation_documents_mut()
        .attach(graph)
        .expect("document must attach once");

    let prepared = context
        .animation_documents()
        .prepare_mutation(
            document,
            &AnimationDocumentMutation::RemoveGraphNode {
                node_id: "stale_node".to_string(),
            },
        )
        .expect("reference cleanup must prepare");

    assert!(
        prepared.is_some(),
        "clearing a persisted dangling reference is a source mutation even when no node row exists"
    );
}

#[test]
fn document_compilation_retains_last_known_good_across_invalid_intermediate_edits() {
    let document = DocumentId::new(44);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(valid_graph_document(document))
        .expect("valid graph document must attach");
    let mutation = AnimationDocumentMutation::RemoveGraphNode {
        node_id: "idle".to_string(),
    };
    let (expected_revision, replacement) = context
        .animation_documents()
        .prepare_mutation(document, &mutation)
        .expect("mutation must prepare")
        .expect("removing an output source node must change source");
    let engine = EditorTransactionEngine::new(context);
    let mut transaction = engine
        .begin(mutation.label(), HistoryContextId::Document(document))
        .expect("document transaction must begin");
    transaction
        .push(AnimationEditCommand::new(
            mutation.label(),
            document,
            expected_revision,
            replacement,
        ))
        .expect("command must apply");
    transaction.commit().expect("transaction must commit");

    let after_invalid_edit = engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(document).unwrap();
            let document = handle.read();
            let compilation = document.compilation();
            (
                document.revision(),
                compilation.current_revision(),
                compilation.current_product().is_successful(),
                compilation.last_good_revision(),
                compilation
                    .last_good_product()
                    .is_some_and(|product| product.is_successful()),
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(after_invalid_edit.0.value(), 2);
    assert_eq!(after_invalid_edit.1.value(), 2);
    assert!(!after_invalid_edit.2);
    assert_eq!(
        after_invalid_edit.3.map(AnimationDocumentRevision::value),
        Some(1)
    );
    assert!(after_invalid_edit.4);

    assert!(engine
        .undo(HistoryContextId::Document(document))
        .expect("undo must succeed"));
    let after_undo = engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(document).unwrap();
            let document = handle.read();
            let compilation = document.compilation();
            (
                compilation.current_product().is_successful(),
                compilation.last_good_revision(),
            )
        })
        .unwrap()
        .unwrap();
    assert!(after_undo.0);
    assert_eq!(after_undo.1.map(AnimationDocumentRevision::value), Some(3));
}
