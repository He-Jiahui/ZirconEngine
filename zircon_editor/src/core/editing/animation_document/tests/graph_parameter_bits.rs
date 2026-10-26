use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationParameterValue,
};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationEditCommand,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

fn numeric(values: &[f32]) -> AnimationParameterValue {
    match values {
        [value] => AnimationParameterValue::Scalar(*value),
        [x, y] => AnimationParameterValue::Vec2([*x, *y]),
        [x, y, z] => AnimationParameterValue::Vec3([*x, *y, *z]),
        [x, y, z, w] => AnimationParameterValue::Vec4([*x, *y, *z, *w]),
        _ => panic!("parameter fixture must have one to four components"),
    }
}

fn signed_zero_cases() -> Vec<(AnimationParameterValue, String, AnimationParameterValue)> {
    let mut cases = Vec::new();
    for width in 1..=4 {
        for component in 0..width {
            for (before_zero, after_zero) in [(-0.0_f32, 0.0_f32), (0.0, -0.0)] {
                let mut before = [1.25_f32, -2.5, 3.75, -4.5][..width].to_vec();
                before[component] = before_zero;
                let mut after = before.clone();
                after[component] = after_zero;
                let literal = after
                    .iter()
                    .map(f32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                cases.push((numeric(&before), literal, numeric(&after)));
            }
        }
    }
    cases
}

fn source(value: AnimationParameterValue) -> AnimationAuthoringAsset {
    AnimationAuthoringAsset::Graph(AnimationGraphAsset {
        name: Some("Authored parameter bits".to_string()),
        parameters: vec![
            AnimationGraphParameterAsset {
                name: "keep_integer".to_string(),
                default_value: AnimationParameterValue::Integer(i32::MIN + 10),
            },
            AnimationGraphParameterAsset {
                name: "selected".to_string(),
                default_value: value,
            },
            AnimationGraphParameterAsset {
                name: "keep_scalar".to_string(),
                default_value: AnimationParameterValue::Scalar(f32::from_bits(1)),
            },
        ],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: AssetReference::from_locator(
                    AssetUri::parse("res://animation/parameter_bits.clip").unwrap(),
                ),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Output {
                source: "idle".to_string(),
            },
        ],
    })
}

fn change(literal: &str) -> AnimationDocumentMutation {
    AnimationDocumentMutation::SetGraphParameter {
        parameter_name: "selected".to_string(),
        value_literal: literal.to_string(),
    }
}

fn engine_with_source(id: DocumentId, asset: AnimationAuthoringAsset) -> EditorTransactionEngine {
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/parameter_bits.graph.zranim").unwrap(),
            asset,
        ))
        .unwrap();
    EditorTransactionEngine::new(context)
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
            let bytes = document.document_bytes().unwrap();
            assert_eq!(bytes, expected.to_bytes().unwrap());
            let reopened = AnimationAuthoringAsset::from_bytes(document.kind(), &bytes).unwrap();
            assert_eq!(reopened.to_bytes().unwrap(), bytes);
            assert_eq!(
                document.compilation().current_revision(),
                document.revision()
            );
            assert!(document.compilation().current_product().is_successful());
            assert_eq!(
                document.compilation().last_good_revision(),
                Some(document.revision())
            );
        })
        .unwrap()
        .unwrap();
}

#[test]
fn graph_parameter_bits_sign_only_edits_keep_all_components_and_codec_bits() {
    let cases = signed_zero_cases();
    assert_eq!(cases.len(), 20);
    for (before, literal, after) in cases {
        let mut asset = source(before);
        let original_bytes = asset.to_bytes().unwrap();
        let expected = source(after);
        assert!(asset.compile().is_successful());
        assert!(change(&literal).apply(&mut asset).unwrap());
        assert_ne!(asset.to_bytes().unwrap(), original_bytes);
        assert_eq!(asset.to_bytes().unwrap(), expected.to_bytes().unwrap());
        let reopened =
            AnimationAuthoringAsset::from_bytes(asset.kind(), &asset.to_bytes().unwrap()).unwrap();
        assert_eq!(reopened.to_bytes().unwrap(), expected.to_bytes().unwrap());
        assert!(!change(&literal).apply(&mut asset).unwrap());
        assert_eq!(asset.to_bytes().unwrap(), expected.to_bytes().unwrap());
    }
}

#[test]
fn graph_parameter_bits_sign_only_edit_uses_one_document_history_and_exact_undo_redo() {
    for (before, literal, after) in signed_zero_cases() {
        let id = DocumentId::new(9194);
        let original = source(before);
        let expected = source(after);
        let engine = engine_with_source(id, original.clone());
        let history = HistoryContextId::Document(id);
        let mutation = change(&literal);
        let before_history = engine.history_status(history).unwrap();
        let (revision, replacement) = engine
            .with_context::<CoreEditContext, _>(|context| {
                context
                    .animation_documents()
                    .prepare_mutation(id, &mutation)
                    .unwrap()
                    .expect("a sign-only source change must prepare a replacement")
            })
            .unwrap()
            .unwrap();
        assert_eq!(
            replacement.to_bytes().unwrap(),
            expected.to_bytes().unwrap()
        );
        assert_document(&engine, id, 1, &original);
        assert!(!engine.is_dirty(history).unwrap());
        assert_eq!(engine.history_status(history).unwrap(), before_history);

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
        assert_document(&engine, id, 2, &expected);
        assert!(engine.is_dirty(history).unwrap());
        assert!(engine.undo(history).unwrap());
        assert_document(&engine, id, 3, &original);
        assert!(!engine.is_dirty(history).unwrap());
        assert!(!engine.undo(history).unwrap());
        assert!(engine.redo(history).unwrap());
        assert_document(&engine, id, 4, &expected);
        assert!(!engine.redo(history).unwrap());
        assert!(engine.is_dirty(history).unwrap());
    }
}

#[test]
fn graph_parameter_bits_exact_noop_or_invalid_literal_keeps_source_and_history_clean() {
    for (before, _, _) in signed_zero_cases() {
        let id = DocumentId::new(9195);
        let original = source(before);
        let engine = engine_with_source(id, original.clone());
        let history = HistoryContextId::Document(id);
        let before_history = engine.history_status(history).unwrap();
        let value = &original.as_graph().unwrap().parameters[1].default_value;
        let exact_literal = match value {
            AnimationParameterValue::Scalar(value) => value.to_string(),
            AnimationParameterValue::Vec2(values) => {
                values.map(|value| value.to_string()).join(",")
            }
            AnimationParameterValue::Vec3(values) => {
                values.map(|value| value.to_string()).join(",")
            }
            AnimationParameterValue::Vec4(values) => {
                values.map(|value| value.to_string()).join(",")
            }
            _ => panic!("signed-zero cases must have numeric parameters"),
        };
        for literal in [exact_literal.as_str(), "NaN", "inf", "1, NaN", "true", ""] {
            let mut direct = original.clone();
            assert!(!change(literal).apply(&mut direct).unwrap());
            assert_eq!(direct.to_bytes().unwrap(), original.to_bytes().unwrap());
            engine
                .with_context::<CoreEditContext, _>(|context| {
                    assert!(context
                        .animation_documents()
                        .prepare_mutation(id, &change(literal))
                        .unwrap()
                        .is_none());
                })
                .unwrap()
                .unwrap();
            assert_document(&engine, id, 1, &original);
            assert!(!engine.is_dirty(history).unwrap());
            assert_eq!(engine.history_status(history).unwrap(), before_history);
            assert!(!engine.undo(history).unwrap());
        }
    }
}
