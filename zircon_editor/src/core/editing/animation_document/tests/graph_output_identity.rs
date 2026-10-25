use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{AnimationGraphAsset, AnimationGraphNodeAsset};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationEditCommand, AnimationGraphNodeKind,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

fn clip(id: &str) -> AnimationGraphNodeAsset {
    AnimationGraphNodeAsset::Clip {
        id: id.to_string(),
        clip: AssetReference::from_locator(
            AssetUri::parse("res://animation/identity.clip").unwrap(),
        ),
        playback_speed: 1.0,
        looping: true,
    }
}

fn graph(with_output: bool) -> AnimationGraphAsset {
    let mut nodes = vec![clip("idle")];
    if with_output {
        nodes.push(AnimationGraphNodeAsset::Output {
            source: "idle".to_string(),
        });
    }
    AnimationGraphAsset {
        name: Some("Output identity".to_string()),
        parameters: Vec::new(),
        nodes,
    }
}

#[test]
fn graph_output_identity_rejects_new_named_sink_alias_without_source_change() {
    for with_output in [false, true] {
        let original = graph(with_output);
        let original_bytes = original.to_bytes().unwrap();
        let mut asset = original.clone();
        assert!(!super::add_node(&mut asset, "output", AnimationGraphNodeKind::Blend).unwrap());
        assert_eq!(asset, original);
        assert_eq!(asset.to_bytes().unwrap(), original_bytes);
    }
}

#[test]
fn graph_output_identity_rejects_sink_after_each_existing_named_alias_kind() {
    for named in [
        clip("output"),
        AnimationGraphNodeAsset::Blend {
            id: "output".to_string(),
            inputs: vec!["idle".to_string()],
            weight_parameter: None,
        },
        AnimationGraphNodeAsset::Additive {
            id: "output".to_string(),
            base: "idle".to_string(),
            additive: "idle".to_string(),
            weight_parameter: None,
        },
        AnimationGraphNodeAsset::Mask {
            id: "output".to_string(),
            input: "idle".to_string(),
            target_ids: vec!["9007199254740993".to_string()],
        },
    ] {
        let mut asset = graph(false);
        asset.nodes.push(named);
        let original = asset.clone();
        let original_bytes = asset.to_bytes().unwrap();
        assert!(!super::add_node(&mut asset, "output", AnimationGraphNodeKind::Output).unwrap());
        assert_eq!(asset, original);
        assert_eq!(asset.to_bytes().unwrap(), original_bytes);
    }
}

#[test]
fn graph_output_identity_keeps_legacy_codec_and_distinct_named_node_commands() {
    // Source format permits a named `output`; creation guards do not rewrite legacy assets.
    let legacy = AnimationGraphAsset {
        name: Some("Legacy named output".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            clip("output"),
            AnimationGraphNodeAsset::Output {
                source: "output".to_string(),
            },
        ],
    };
    let reopened = AnimationGraphAsset::from_bytes(&legacy.to_bytes().unwrap()).unwrap();
    assert_eq!(reopened, legacy);
    assert!(AnimationAuthoringAsset::Graph(reopened)
        .compile()
        .is_successful());

    for id in ["mix", "Output", "output2"] {
        let mut asset = graph(true);
        assert!(super::add_node(&mut asset, id, AnimationGraphNodeKind::Blend).unwrap());
        assert!(!super::add_node(&mut asset, id, AnimationGraphNodeKind::Blend).unwrap());
        assert!(!super::add_node(&mut asset, "output", AnimationGraphNodeKind::Output).unwrap());
        assert!(super::connect_nodes(&mut asset, "idle", id));
        assert!(super::connect_nodes(&mut asset, id, "output"));
        assert!(AnimationAuthoringAsset::Graph(asset.clone())
            .compile()
            .is_successful());
        assert_eq!(
            AnimationGraphAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap(),
            asset
        );
        assert!(super::remove_node(&mut asset, id));
        let mut expected = graph(true);
        expected.nodes[1] = AnimationGraphNodeAsset::Output {
            source: String::new(),
        };
        assert_eq!(asset, expected);
    }
}

fn assert_document(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    revision: u64,
    expected: &AnimationGraphAsset,
    successful: bool,
    last_good_revision: u64,
) {
    engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(id).unwrap();
            let document = handle.read();
            assert_eq!(document.revision().value(), revision);
            assert_eq!(document.asset().as_graph(), Some(expected));
            let bytes = document.document_bytes().unwrap();
            assert_eq!(bytes, expected.to_bytes().unwrap());
            assert_eq!(AnimationGraphAsset::from_bytes(&bytes).unwrap(), *expected);
            let compilation = document.compilation();
            assert_eq!(compilation.current_revision(), document.revision());
            assert_eq!(compilation.current_product().is_successful(), successful);
            assert_eq!(
                compilation.last_good_revision().map(|value| value.value()),
                Some(last_good_revision)
            );
            assert!(compilation.last_good_product().unwrap().is_successful());
        })
        .unwrap()
        .unwrap();
}

#[test]
fn graph_output_identity_rejection_and_valid_edits_use_original_document_history() {
    let id = DocumentId::new(9184);
    let original = graph(true);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/identity.graph.zranim").unwrap(),
            AnimationAuthoringAsset::Graph(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let before = engine.history_status(history).unwrap();
    assert!(engine
        .with_context::<CoreEditContext, _>(|context| {
            context
                .animation_documents()
                .prepare_mutation(
                    id,
                    &AnimationDocumentMutation::AddGraphNode {
                        node_id: "output".to_string(),
                        node_kind: AnimationGraphNodeKind::Blend,
                    },
                )
                .unwrap()
                .is_none()
        })
        .unwrap()
        .unwrap());
    assert_eq!(engine.history_status(history).unwrap(), before);
    assert_document(&engine, id, 1, &original, true, 1);
    assert!(!engine.is_dirty(history).unwrap());

    let apply = |mutation: AnimationDocumentMutation| {
        let (revision, replacement) = engine
            .with_context::<CoreEditContext, _>(|context| {
                context
                    .animation_documents()
                    .prepare_mutation(id, &mutation)
                    .unwrap()
                    .unwrap()
            })
            .unwrap()
            .unwrap();
        let mut transaction = engine.begin(mutation.label(), history).unwrap();
        transaction
            .push(AnimationEditCommand::new(
                mutation.label(),
                id,
                revision,
                replacement,
            ))
            .unwrap();
        transaction.commit().unwrap();
    };
    let mut added = original.clone();
    assert!(super::add_node(&mut added, "mix", AnimationGraphNodeKind::Blend).unwrap());
    let mut connected = added.clone();
    assert!(super::connect_nodes(&mut connected, "idle", "mix"));
    let mut final_graph = connected.clone();
    assert!(super::connect_nodes(&mut final_graph, "mix", "output"));

    apply(AnimationDocumentMutation::AddGraphNode {
        node_id: "mix".to_string(),
        node_kind: AnimationGraphNodeKind::Blend,
    });
    assert_document(&engine, id, 2, &added, false, 1);
    assert!(engine.is_dirty(history).unwrap());
    apply(AnimationDocumentMutation::ConnectGraphNodes {
        from_node_id: "idle".to_string(),
        to_node_id: "mix".to_string(),
    });
    assert_document(&engine, id, 3, &connected, true, 3);
    apply(AnimationDocumentMutation::ConnectGraphNodes {
        from_node_id: "mix".to_string(),
        to_node_id: "output".to_string(),
    });
    assert_document(&engine, id, 4, &final_graph, true, 4);
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 5, &connected, true, 5);
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 6, &added, false, 5);
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 7, &original, true, 7);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(!engine.undo(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 8, &added, false, 7);
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 9, &connected, true, 9);
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 10, &final_graph, true, 10);
    assert!(engine.is_dirty(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}

fn legacy_ambiguous_graph() -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("Legacy ambiguous output".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            clip("idle"),
            clip("output"),
            AnimationGraphNodeAsset::Blend {
                id: "mix".to_string(),
                inputs: vec!["idle".to_string()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output {
                source: "output".to_string(),
            },
        ],
    }
}

fn assert_legacy_admission_rejected(
    graph: &AnimationGraphAsset,
    mutation: &AnimationDocumentMutation,
) {
    let before = graph.to_bytes().unwrap();
    let mut candidate = AnimationAuthoringAsset::Graph(graph.clone());
    let error = mutation.apply(&mut candidate).unwrap_err();
    assert!(matches!(
        error,
        crate::core::editing::animation_document::AnimationDocumentMutationError::InvalidGraphTarget {
            ..
        }
    ));
    assert_eq!(candidate.as_graph(), Some(graph));
    assert_eq!(candidate.as_graph().unwrap().to_bytes().unwrap(), before);
    assert_eq!(AnimationGraphAsset::from_bytes(&before).unwrap(), *graph);
}

#[test]
fn graph_legacy_admission_rejects_named_output_sink_alias_before_source_writes() {
    let mut original = legacy_ambiguous_graph();
    original.nodes[1] = AnimationGraphNodeAsset::Blend {
        id: "output".to_string(),
        inputs: vec!["idle".to_string()],
        weight_parameter: None,
    };
    assert!(AnimationAuthoringAsset::Graph(original.clone())
        .compile()
        .is_successful());
    for mutation in [
        AnimationDocumentMutation::RemoveGraphNode {
            node_id: "output".to_string(),
        },
        AnimationDocumentMutation::ConnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "output".to_string(),
        },
        AnimationDocumentMutation::DisconnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "output".to_string(),
        },
    ] {
        assert_legacy_admission_rejected(&original, &mutation);
    }
}

#[test]
fn graph_legacy_admission_rejects_duplicate_named_targets_and_sources() {
    let mut original = graph(true);
    original.nodes.push(clip("mix"));
    original.nodes.push(AnimationGraphNodeAsset::Blend {
        id: "mix".to_string(),
        inputs: vec!["idle".to_string()],
        weight_parameter: None,
    });
    assert!(!AnimationAuthoringAsset::Graph(original.clone())
        .compile()
        .is_successful());
    for mutation in [
        AnimationDocumentMutation::RemoveGraphNode {
            node_id: "mix".to_string(),
        },
        AnimationDocumentMutation::ConnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "mix".to_string(),
        },
        AnimationDocumentMutation::DisconnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "mix".to_string(),
        },
        AnimationDocumentMutation::ConnectGraphNodes {
            from_node_id: "mix".to_string(),
            to_node_id: "output".to_string(),
        },
        AnimationDocumentMutation::DisconnectGraphNodes {
            from_node_id: "mix".to_string(),
            to_node_id: "output".to_string(),
        },
    ] {
        assert_legacy_admission_rejected(&original, &mutation);
    }
}

#[test]
fn graph_legacy_admission_rejects_multiple_output_sinks_without_normalizing_source() {
    let mut original = graph(true);
    original.nodes.push(AnimationGraphNodeAsset::Output {
        source: "idle".to_string(),
    });
    assert!(!AnimationAuthoringAsset::Graph(original.clone())
        .compile()
        .is_successful());
    for mutation in [
        AnimationDocumentMutation::RemoveGraphNode {
            node_id: "output".to_string(),
        },
        AnimationDocumentMutation::ConnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "output".to_string(),
        },
        AnimationDocumentMutation::DisconnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "output".to_string(),
        },
    ] {
        assert_legacy_admission_rejected(&original, &mutation);
    }
}

#[test]
fn graph_legacy_admission_keeps_named_source_role_and_unambiguous_targets() {
    let mut asset = AnimationAuthoringAsset::Graph(legacy_ambiguous_graph());
    assert!(AnimationDocumentMutation::ConnectGraphNodes {
        from_node_id: "output".to_string(),
        to_node_id: "mix".to_string(),
    }
    .apply(&mut asset)
    .unwrap());
    assert_eq!(
        asset.as_graph().unwrap().nodes[2],
        AnimationGraphNodeAsset::Blend {
            id: "mix".to_string(),
            inputs: vec!["idle".to_string(), "output".to_string()],
            weight_parameter: None,
        }
    );
    assert!(AnimationDocumentMutation::DisconnectGraphNodes {
        from_node_id: "output".to_string(),
        to_node_id: "mix".to_string(),
    }
    .apply(&mut asset)
    .unwrap());
    assert_eq!(asset.as_graph(), Some(&legacy_ambiguous_graph()));

    // Without a sink, the old target string identifies one existing named node.
    let mut named_only = graph(false);
    named_only.nodes.push(AnimationGraphNodeAsset::Blend {
        id: "output".to_string(),
        inputs: Vec::new(),
        weight_parameter: None,
    });
    let mut named_only = AnimationAuthoringAsset::Graph(named_only);
    assert!(AnimationDocumentMutation::ConnectGraphNodes {
        from_node_id: "idle".to_string(),
        to_node_id: "output".to_string(),
    }
    .apply(&mut named_only)
    .unwrap());
    assert!(AnimationDocumentMutation::DisconnectGraphNodes {
        from_node_id: "idle".to_string(),
        to_node_id: "output".to_string(),
    }
    .apply(&mut named_only)
    .unwrap());
    assert!(AnimationDocumentMutation::RemoveGraphNode {
        node_id: "output".to_string(),
    }
    .apply(&mut named_only)
    .unwrap());
    assert_eq!(named_only.as_graph(), Some(&graph(false)));

    let mut sink_only = AnimationAuthoringAsset::Graph(graph(true));
    assert!(AnimationDocumentMutation::DisconnectGraphNodes {
        from_node_id: "idle".to_string(),
        to_node_id: "output".to_string(),
    }
    .apply(&mut sink_only)
    .unwrap());
    assert!(AnimationDocumentMutation::ConnectGraphNodes {
        from_node_id: "idle".to_string(),
        to_node_id: "output".to_string(),
    }
    .apply(&mut sink_only)
    .unwrap());
    assert_eq!(sink_only.as_graph(), Some(&graph(true)));
    assert!(AnimationDocumentMutation::RemoveGraphNode {
        node_id: "output".to_string(),
    }
    .apply(&mut sink_only)
    .unwrap());
    assert_eq!(sink_only.as_graph(), Some(&graph(false)));
}

#[test]
fn graph_legacy_admission_refusal_preserves_original_document_and_history_authority() {
    let id = DocumentId::new(9185);
    let original = legacy_ambiguous_graph();
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/legacy.graph.zranim").unwrap(),
            AnimationAuthoringAsset::Graph(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let before = engine.history_status(history).unwrap();
    for mutation in [
        AnimationDocumentMutation::RemoveGraphNode {
            node_id: "output".to_string(),
        },
        AnimationDocumentMutation::ConnectGraphNodes {
            from_node_id: "idle".to_string(),
            to_node_id: "output".to_string(),
        },
        AnimationDocumentMutation::DisconnectGraphNodes {
            from_node_id: "output".to_string(),
            to_node_id: "output".to_string(),
        },
    ] {
        engine
            .with_context::<CoreEditContext, _>(|context| {
                let result = context.animation_documents().prepare_mutation(id, &mutation);
                assert!(matches!(
                    result,
                    Err(crate::core::editing::animation_document::AnimationDocumentMutationError::InvalidGraphTarget {
                        ..
                    })
                ));
            })
            .unwrap()
            .unwrap();
        assert_eq!(engine.history_status(history).unwrap(), before);
        assert!(!engine.is_dirty(history).unwrap());
        assert_document(&engine, id, 1, &original, true, 1);
    }
    assert!(!engine.undo(history).unwrap());
    assert!(!engine.redo(history).unwrap());

    // A non-ambiguous edit still uses the original command/history, preserving both output roles.
    let mutation = AnimationDocumentMutation::ConnectGraphNodes {
        from_node_id: "output".to_string(),
        to_node_id: "mix".to_string(),
    };
    let (revision, replacement) = engine
        .with_context::<CoreEditContext, _>(|context| {
            context
                .animation_documents()
                .prepare_mutation(id, &mutation)
        })
        .unwrap()
        .unwrap()
        .unwrap()
        .unwrap();
    let mut transaction = engine.begin(mutation.label(), history).unwrap();
    transaction
        .push(AnimationEditCommand::new(
            mutation.label(),
            id,
            revision,
            replacement,
        ))
        .unwrap();
    transaction.commit().unwrap();
    let mut connected = original.clone();
    match &mut connected.nodes[2] {
        AnimationGraphNodeAsset::Blend { inputs, .. } => inputs.push("output".to_string()),
        _ => panic!("expected original named blend"),
    }
    assert_document(&engine, id, 2, &connected, true, 2);
    assert!(engine.is_dirty(history).unwrap());
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 3, &original, true, 3);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 4, &connected, true, 4);
    assert!(engine.is_dirty(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}
