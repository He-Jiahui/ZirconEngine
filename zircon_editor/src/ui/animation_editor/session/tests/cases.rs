use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationTrackPath,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentReadHandle,
    AnimationDocumentMutation, AnimationEditCommand,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

use super::AnimationEditorSession;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    PathBuf::from("E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-session-tests")
        .join(format!("{prefix}_{unique}"))
}

fn write_sequence_asset(path: &Path) -> AnimationTrackPath {
    let track_path = AnimationTrackPath::new(
        EntityPath::parse("Root/Hero").expect("fixture entity path must be valid"),
        ComponentPropertyPath::parse("Transform.translation")
            .expect("fixture property path must be valid"),
    );
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([1.0, 2.0, 3.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 1.0, 2.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([3.0, 4.0, 5.0])),
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 2.0,
                            value: AnimationChannelValueAsset::Vec3([4.0, 5.0, 6.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([6.0, 7.0, 8.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([9.0, 10.0, 11.0])),
                        },
                    ],
                },
            }],
        }],
    };
    fs::create_dir_all(path.parent().expect("fixture path must have a parent"))
        .expect("fixture directory must be created");
    fs::write(path, asset.to_bytes().expect("fixture must encode"))
        .expect("fixture bytes must write");
    track_path
}

#[test]
fn sequence_selection_is_transient_and_does_not_mutate_core_source_bytes() {
    let path = unique_temp_dir("zircon_animation_session_selection").join("hero.sequence.zranim");
    let track_path = write_sequence_asset(&path);
    let mut session = AnimationEditorSession::from_path(&path).expect("fixture must open");
    let before = session.document_bytes().expect("source must serialize");

    assert!(session
        .select_timeline_span(&track_path, 4, 28)
        .expect("selection must apply"));

    assert_eq!(
        session.document_bytes().expect("source must serialize"),
        before,
        "selection belongs to the UI session and must not create another persisted source state"
    );
    let _ = fs::remove_dir_all(path.parent().expect("fixture path must have a parent"));
}

#[test]
fn sequence_timeline_projection_reads_core_source_and_transient_transport() {
    let path = unique_temp_dir("zircon_animation_session_timeline").join("hero.sequence.zranim");
    let track_path = write_sequence_asset(&path);
    let mut session = AnimationEditorSession::from_path(&path).expect("fixture must open");
    session.set_timeline_range(10, 20).unwrap();
    session.scrub_timeline(15).unwrap();
    session.set_playback(true, true, 1.25).unwrap();
    session.select_timeline_span(&track_path, 10, 20).unwrap();

    let timeline = session
        .timeline_foundation()
        .expect("timeline must project");
    assert_eq!(timeline.tracks.len(), 1);
    assert!((timeline.playhead - 0.5).abs() < 1.0e-6);
    assert!(timeline.playback.playing);
    assert_eq!(timeline.playback.rate, 1.25);
    let curves = session
        .curve_foundation()
        .expect("curve projection must succeed");
    assert_eq!(curves.curves.len(), 3);
    assert_eq!(curves.curves[2].keys[1].in_tangent, Some(8.0));
    let _ = fs::remove_dir_all(path.parent().expect("fixture path must have a parent"));
}

#[test]
fn non_finite_playback_speed_preserves_transient_transport() {
    let path = unique_temp_dir("zircon_animation_session_playback").join("hero.sequence.zranim");
    write_sequence_asset(&path);
    let mut session = AnimationEditorSession::from_path(&path).expect("fixture must open");

    assert!(!session
        .set_playback(true, true, f32::NAN)
        .expect("invalid speed must be a no-op"));
    assert_eq!(
        session.pane_presentation().playback_label,
        "Paused • loop=false • speed=1.00"
    );
    let _ = fs::remove_dir_all(path.parent().expect("fixture path must have a parent"));
}

fn frame_time_source(frames_per_second: f32) -> (AnimationSequenceAsset, AnimationTrackPath) {
    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let property_path = ComponentPropertyPath::parse("AnimationPlayer.playback_speed").unwrap();
    let track_path = AnimationTrackPath::new(entity_path.clone(), property_path.clone());
    (
        AnimationSequenceAsset {
            name: Some("Frame and second agreement".to_string()),
            duration_seconds: 8.0,
            frames_per_second,
            bindings: vec![AnimationSequenceBindingAsset {
                entity_path,
                target_id: Some("9007199254740993".to_string()),
                tracks: vec![AnimationSequenceTrackAsset {
                    property_path,
                    channel: AnimationChannelAsset {
                        interpolation: AnimationInterpolationAsset::Linear,
                        keys: vec![
                            AnimationChannelKeyAsset {
                                time_seconds: 0.0,
                                value: AnimationChannelValueAsset::Scalar(1.0),
                                in_tangent: None,
                                out_tangent: None,
                            },
                            AnimationChannelKeyAsset {
                                time_seconds: 8.0,
                                value: AnimationChannelValueAsset::Scalar(2.0),
                                in_tangent: None,
                                out_tangent: None,
                            },
                        ],
                    },
                }],
            }],
        },
        track_path,
    )
}

fn frame_time_session(asset: AnimationSequenceAsset) -> AnimationEditorSession {
    let document = AnimationAuthoringDocument::new(
        DocumentId::new(9183),
        AssetUri::parse("res://tests/frame-time.sequence.zranim").unwrap(),
        AnimationAuthoringAsset::Sequence(asset),
    );
    AnimationEditorSession::new(
        "res://tests/frame-time.sequence.zranim".to_string(),
        AnimationAuthoringDocumentReadHandle::detached_for_test(document),
    )
}

#[test]
fn sequence_frame_time_projection_keeps_fractional_fps_seconds() {
    for fps in [0.25, 0.5, 1.0, 30.0] {
        let (asset, _) = frame_time_source(fps);
        let bytes = asset.to_bytes().unwrap();
        let mut session = frame_time_session(asset);
        assert!(session.scrub_timeline(1).unwrap());
        session.set_playback(true, true, 1.75).unwrap();
        let timeline = session.timeline_foundation().unwrap();

        assert_eq!(timeline.range.start, 0.0);
        assert_eq!(timeline.range.end, 8.0);
        assert_eq!(timeline.playhead, 1.0 / fps);
        assert_eq!(timeline.playback.time_seconds, timeline.playhead);
        assert_eq!(timeline.playback.rate, 1.75);
        assert_eq!(timeline.tracks[0].keys[1].time, timeline.range.end);
        assert_eq!(session.document_bytes().unwrap(), bytes);
    }
}

#[test]
fn sequence_frame_time_key_mutations_use_same_seconds_as_projection() {
    for fps in [0.25, 0.5, 30.0] {
        let (original, track_path) = frame_time_source(fps);
        let mut source = AnimationAuthoringAsset::Sequence(original.clone());
        assert!(AnimationDocumentMutation::AddKey {
            track_path: track_path.clone(),
            frame: 1,
        }
        .apply(&mut source)
        .unwrap());
        let added = source.as_sequence().unwrap();
        assert_eq!(
            added.bindings[0].tracks[0].channel.keys[1].time_seconds,
            1.0 / fps
        );
        assert!(source.compile().is_successful());

        let reopened = AnimationSequenceAsset::from_bytes(&source.to_bytes().unwrap()).unwrap();
        assert_eq!(&reopened, added);
        let mut session = frame_time_session(reopened);
        session.scrub_timeline(1).unwrap();
        let timeline = session.timeline_foundation().unwrap();
        assert_eq!(timeline.tracks[0].keys[1].time, timeline.playhead);

        assert!(AnimationDocumentMutation::RemoveKey {
            track_path,
            frame: 1,
        }
        .apply(&mut source)
        .unwrap());
        assert_eq!(source.as_sequence(), Some(&original));
    }
}

#[test]
fn sequence_frame_time_history_uses_same_seconds_after_undo_redo() {
    let id = DocumentId::new(9184);
    let (original, track_path) = frame_time_source(0.5);
    let original_bytes = original.to_bytes().unwrap();
    let mut context = CoreEditContext::default();
    let handle = context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://tests/frame-history.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original),
        ))
        .unwrap();
    let mut session = AnimationEditorSession::new(
        "res://tests/frame-history.sequence.zranim".to_string(),
        handle,
    );
    session.scrub_timeline(1).unwrap();
    let mutation = AnimationDocumentMutation::AddKey {
        track_path,
        frame: 1,
    };
    let (revision, replacement) = context
        .animation_documents()
        .prepare_mutation(id, &mutation)
        .unwrap()
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
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

    let timeline = session.timeline_foundation().unwrap();
    assert_eq!(timeline.range.end, 8.0);
    assert_eq!(timeline.playhead, 2.0);
    assert_eq!(timeline.tracks[0].keys[1].time, 2.0);
    let committed = session.document_bytes().unwrap();
    let reopened = AnimationSequenceAsset::from_bytes(&committed).unwrap();
    assert_eq!(reopened.frames_per_second, 0.5);
    assert_eq!(
        reopened.bindings[0].tracks[0].channel.keys[1].time_seconds,
        2.0
    );
    assert!(session
        .document()
        .read()
        .compilation()
        .current_product()
        .is_successful());
    assert!(engine.is_dirty(history).unwrap());

    assert!(engine.undo(history).unwrap());
    assert_eq!(session.document_bytes().unwrap(), original_bytes);
    assert_eq!(session.timeline_foundation().unwrap().playhead, 2.0);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_eq!(session.document_bytes().unwrap(), committed);
    assert_eq!(
        session.timeline_foundation().unwrap().tracks[0].keys[1].time,
        2.0
    );
}

#[test]
fn sequence_frame_time_invalid_fps_rejects_mutation_and_preview_projection() {
    for fps in [
        0.0,
        -0.0,
        -1.0,
        f32::EPSILON,
        f32::MIN_POSITIVE,
        f32::from_bits(1),
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ] {
        let (asset, track_path) = frame_time_source(fps);
        let mut source = AnimationAuthoringAsset::Sequence(asset.clone());
        let bytes = source.to_bytes().unwrap();
        for mutation in [
            AnimationDocumentMutation::AddKey {
                track_path: track_path.clone(),
                frame: 30,
            },
            AnimationDocumentMutation::RemoveKey {
                track_path: track_path.clone(),
                frame: 0,
            },
        ] {
            let error = mutation.apply(&mut source).unwrap_err().to_string();
            assert!(error.contains("frames_per_second"));
            assert_eq!(source.to_bytes().unwrap(), bytes);
        }
        assert!(!source.compile().is_successful());
        let mut session = frame_time_session(asset);
        session.scrub_timeline(30).unwrap();
        session.set_playback(true, true, 1.0).unwrap();
        let before = session.pane_presentation();
        for _ in 0..2 {
            let error = session.timeline_foundation().unwrap_err().to_string();
            assert!(error.contains("frames_per_second"));
            assert_eq!(session.document_bytes().unwrap(), bytes);
            let after = session.pane_presentation();
            assert_eq!(after.current_frame, before.current_frame);
            assert_eq!(after.timeline_end_frame, before.timeline_end_frame);
            assert_eq!(after.playback_label, before.playback_label);
        }
        let document = session.document().read();
        assert_eq!(
            document
                .asset()
                .as_sequence()
                .unwrap()
                .frames_per_second
                .to_bits(),
            fps.to_bits()
        );
        assert!(!document.compilation().current_product().is_successful());
    }
}

#[test]
fn sequence_frame_time_projection_rechecks_current_revision_after_undo_redo() {
    let (original, _) = frame_time_source(30.0);
    let original_bytes = original.to_bytes().unwrap();
    let id = DocumentId::new(9189);
    let history = HistoryContextId::Document(id);
    let mut context = CoreEditContext::default();
    let handle = context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://tests/frame-admission.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let mut session = AnimationEditorSession::new(
        "res://tests/frame-admission.sequence.zranim".to_string(),
        handle,
    );
    session.scrub_timeline(30).unwrap();
    session.set_playback(true, true, 1.25).unwrap();
    let original_view = session.timeline_foundation().unwrap();
    let original_revision = session.document().read().revision();
    let mut invalid = original;
    invalid.frames_per_second = 0.0;
    let invalid_bytes = invalid.to_bytes().unwrap();
    let engine = EditorTransactionEngine::new(context);
    let mut transaction = engine.begin("Set invalid source rate", history).unwrap();
    transaction
        .push(AnimationEditCommand::new(
            "Set invalid source rate",
            id,
            original_revision,
            AnimationAuthoringAsset::Sequence(invalid),
        ))
        .unwrap();
    transaction.commit().unwrap();
    let invalid_revision = session.document().read().revision();
    assert!(invalid_revision > original_revision);
    let committed_history = engine.history_status(history).unwrap();

    for _ in 0..2 {
        assert!(session
            .timeline_foundation()
            .unwrap_err()
            .to_string()
            .contains("frames_per_second"));
        assert_eq!(session.document_bytes().unwrap(), invalid_bytes);
        let document = session.document().read();
        assert_eq!(document.revision(), invalid_revision);
        assert!(!document.compilation().current_product().is_successful());
        assert_eq!(
            document.compilation().last_good_revision(),
            Some(original_revision)
        );
    }
    assert_eq!(engine.history_status(history).unwrap(), committed_history);
    assert!(engine.is_dirty(history).unwrap());

    assert!(engine.undo(history).unwrap());
    let undo_revision = session.document().read().revision();
    assert!(undo_revision > invalid_revision);
    assert_eq!(session.document_bytes().unwrap(), original_bytes);
    assert_eq!(session.timeline_foundation().unwrap(), original_view);
    assert!(!engine.is_dirty(history).unwrap());

    assert!(engine.redo(history).unwrap());
    assert!(session.document().read().revision() > undo_revision);
    assert_eq!(session.document_bytes().unwrap(), invalid_bytes);
    assert!(session.timeline_foundation().is_err());
    assert!(engine.is_dirty(history).unwrap());
}

fn curve_binding_lookup_source(
    leading_empty: bool,
    target_id: Option<&str>,
) -> (AnimationSequenceAsset, AnimationTrackPath) {
    let (mut asset, path) = frame_time_source(30.0);
    asset.bindings[0].target_id = target_id.map(str::to_string);
    let mut leading_track = asset.bindings[0].tracks[0].clone();
    leading_track.property_path = ComponentPropertyPath::parse("AnimationPlayer.time").unwrap();
    let mut other_entity_track = asset.bindings[0].tracks[0].clone();
    for key in &mut other_entity_track.channel.keys {
        key.value = AnimationChannelValueAsset::Scalar(99.0);
    }
    asset.bindings.insert(
        0,
        AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Other").unwrap(),
            target_id: None,
            tracks: vec![other_entity_track],
        },
    );
    asset.bindings.insert(
        0,
        AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: Some("9007199254740995".to_string()),
            tracks: if leading_empty {
                Vec::new()
            } else {
                vec![leading_track]
            },
        },
    );
    (asset, path)
}

#[test]
fn sequence_curve_binding_lookup_projects_the_complete_path_without_source_changes() {
    for target_id in [Some("9007199254740993"), None] {
        for leading_empty in [true, false] {
            let (mut source, path) = curve_binding_lookup_source(leading_empty, target_id);
            let channel = &mut source.bindings[2].tracks[0].channel;
            channel.interpolation = AnimationInterpolationAsset::Hermite;
            for (key, (value, in_tangent, out_tangent)) in channel.keys.iter_mut().zip([
                ([1.0, 2.0, 3.0], [0.0, 1.0, 2.0], [3.0, 4.0, 5.0]),
                ([4.0, 5.0, 6.0], [6.0, 7.0, 8.0], [9.0, 10.0, 11.0]),
            ]) {
                key.value = AnimationChannelValueAsset::Vec3(value);
                key.in_tangent = Some(AnimationChannelValueAsset::Vec3(in_tangent));
                key.out_tangent = Some(AnimationChannelValueAsset::Vec3(out_tangent));
            }
            let bytes = source.to_bytes().unwrap();
            let reopened = AnimationSequenceAsset::from_bytes(&bytes).unwrap();
            assert_eq!(reopened, source);
            assert!(AnimationAuthoringAsset::Sequence(reopened.clone())
                .compile()
                .is_successful());
            let mut session = frame_time_session(reopened);
            assert!(session.select_timeline_span(&path, 0, 30).unwrap());
            let curves = session.curve_foundation().unwrap();
            assert_eq!(curves.selected_track_id, Some(path.to_string()));
            assert_eq!(curves.curves.len(), 3);
            assert_eq!(
                curves.curves[2].interpolation,
                crate::ui::curve::CurveInterpolation::Hermite
            );
            assert_eq!(curves.curves[2].keys[0].point.time, 0.0);
            assert_eq!(curves.curves[2].keys[1].point.time, 8.0);
            assert_eq!(curves.curves[2].keys[1].point.value, 6.0);
            assert_eq!(curves.curves[2].keys[1].in_tangent, Some(8.0));
            assert_eq!(curves.curves[2].keys[1].out_tangent, Some(11.0));
            let timeline = session.timeline_foundation().unwrap();
            let selected = timeline
                .tracks
                .iter()
                .find(|track| track.id == path.to_string())
                .unwrap();
            assert_eq!(selected.value_kind, "vector3");
            assert_eq!(selected.keys[1].time, curves.curves[2].keys[1].point.time);
            assert_eq!(session.document_bytes().unwrap(), bytes);
        }
    }
}

#[test]
fn sequence_curve_binding_lookup_reads_replaced_source_after_undo_redo() {
    let (original, path) = curve_binding_lookup_source(true, Some("9007199254740993"));
    let original_bytes = original.to_bytes().unwrap();
    let id = DocumentId::new(9186);
    let history = HistoryContextId::Document(id);
    let mut context = CoreEditContext::default();
    let handle = context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://tests/curve-binding.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let mut session = AnimationEditorSession::new(
        "res://tests/curve-binding.sequence.zranim".to_string(),
        handle,
    );
    session.select_timeline_span(&path, 0, 30).unwrap();
    let before = session.curve_foundation().unwrap();
    let mutation = AnimationDocumentMutation::AddKey {
        track_path: path.clone(),
        frame: 30,
    };
    let (revision, replacement) = context
        .animation_documents()
        .prepare_mutation(id, &mutation)
        .unwrap()
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
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

    let after = session.curve_foundation().unwrap();
    assert_eq!(after.selected_track_id, before.selected_track_id);
    assert_eq!(after.curves[0].keys.len(), 3);
    assert_eq!(after.curves[0].keys[1].point.time, 1.0);
    let committed = session.document_bytes().unwrap();
    let reopened = AnimationSequenceAsset::from_bytes(&committed).unwrap();
    assert_eq!(reopened.bindings[0], original.bindings[0]);
    assert_eq!(reopened.bindings[1], original.bindings[1]);
    assert_eq!(
        reopened.bindings[2].target_id,
        original.bindings[2].target_id
    );
    assert!(session
        .document()
        .read()
        .compilation()
        .current_product()
        .is_successful());
    assert!(engine.is_dirty(history).unwrap());

    assert!(engine.undo(history).unwrap());
    assert_eq!(session.document_bytes().unwrap(), original_bytes);
    assert_eq!(session.curve_foundation().unwrap(), before);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_eq!(session.document_bytes().unwrap(), committed);
    assert_eq!(session.curve_foundation().unwrap(), after);
}

#[test]
fn sequence_curve_binding_lookup_rejects_ambiguous_targets_without_source_changes() {
    let (mut source, path) = curve_binding_lookup_source(false, Some("9007199254740993"));
    source.bindings[0].tracks[0] = source.bindings[2].tracks[0].clone();
    assert!(AnimationAuthoringAsset::Sequence(source.clone())
        .compile()
        .is_successful());
    let bytes = source.to_bytes().unwrap();
    let mut session = frame_time_session(source);
    let error = session.select_timeline_span(&path, 0, 30).unwrap_err();
    assert!(error.contains("ambiguous animation track"));
    assert!(error.contains(&path.to_string()));
    assert_eq!(session.curve_foundation().unwrap().selected_track_id, None);
    // A stale selection from an earlier source still meets the projection's own guard.
    session.sequence.as_mut().unwrap().selected_span = Some((path.clone(), 0, 30));
    let error = session.curve_foundation().unwrap_err().to_string();
    assert!(error.contains("ambiguous animation track"));
    assert!(error.contains(&path.to_string()));
    assert!(session.select_timeline_span(&path, 0, 30).is_err());
    assert_eq!(session.document_bytes().unwrap(), bytes);
}
