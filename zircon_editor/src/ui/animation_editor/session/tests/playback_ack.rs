use super::*;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::AnimationSequenceAsset;

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentReadHandle,
    AnimationDocumentRevision,
};
use crate::core::editor_message::DocumentId;

fn session() -> AnimationEditorSession {
    let locator = AssetUri::parse("res://tests/sequence-playback-ack.sequence.zranim").unwrap();
    let document = AnimationAuthoringDocument::new(
        DocumentId::new(9191),
        locator,
        AnimationAuthoringAsset::Sequence(AnimationSequenceAsset {
            name: Some("Transient playback receipt".to_string()),
            duration_seconds: 2.0,
            frames_per_second: 30.0,
            bindings: Vec::new(),
        }),
    );
    let mut session = AnimationEditorSession::new(
        "res://tests/sequence-playback-ack.sequence.zranim".to_string(),
        AnimationAuthoringDocumentReadHandle::detached_for_test(document),
    );
    assert!(session.scrub_timeline(15).unwrap());
    session
}

fn source_state(session: &AnimationEditorSession) -> (Vec<u8>, AnimationDocumentRevision, String) {
    let document = session.document().read();
    (
        document.document_bytes().unwrap(),
        document.revision(),
        format!("{:?}", document.compilation()),
    )
}

#[test]
fn sequence_playback_ack_adjacent_finite_rates_match_projected_state() {
    let mut session = session();
    let source = source_state(&session);
    let before = session.timeline_foundation().unwrap();
    for speed in [
        f32::from_bits(1.0_f32.to_bits() + 1),
        1.0,
        0.5,
        f32::from_bits(0.5_f32.to_bits() + 1),
        0.0,
        f32::from_bits(1),
        -f32::from_bits(1),
    ] {
        // The adjacent value above 1.0 differs by exactly EPSILON; both it and smaller
        // finite changes were stored while the original setter returned false.
        assert!(session.set_playback(false, false, speed).unwrap());
        let current = session.timeline_foundation().unwrap();
        assert_eq!(current.playback.rate.to_bits(), speed.to_bits());
        assert_eq!(current.playhead, before.playhead);
        assert_eq!(current.range, before.range);
        assert_eq!(current.tracks, before.tracks);
        assert!(!current.playback.playing);
        assert!(!current.playback.looping);
        assert!(!session.set_playback(false, false, speed).unwrap());
    }
    assert_eq!(source_state(&session), source);
}

#[test]
fn sequence_playback_ack_signed_zero_and_flag_changes_are_reported_once() {
    let mut session = session();
    let source = source_state(&session);
    assert!(session.set_playback(false, false, 0.0).unwrap());
    assert!(session.set_playback(false, false, -0.0).unwrap());
    assert_eq!(
        session
            .timeline_foundation()
            .unwrap()
            .playback
            .rate
            .to_bits(),
        (-0.0_f32).to_bits()
    );
    assert!(!session.set_playback(false, false, -0.0).unwrap());
    assert!(session.set_playback(true, false, -0.0).unwrap());
    assert!(session.set_playback(true, true, -0.0).unwrap());
    assert!(!session.set_playback(true, true, -0.0).unwrap());
    assert!(session.set_playback(true, true, 0.0).unwrap());
    let current = session.timeline_foundation().unwrap();
    assert_eq!(current.playback.rate.to_bits(), 0.0_f32.to_bits());
    assert!(current.playback.playing);
    assert!(current.playback.looping);
    assert_eq!(source_state(&session), source);
}

#[test]
fn sequence_playback_ack_nonfinite_rejects_the_entire_transient_update() {
    let mut session = session();
    assert!(session.set_playback(true, true, 1.75).unwrap());
    let source = source_state(&session);
    let before = session.timeline_foundation().unwrap();
    for invalid in [
        f32::NAN,
        f32::from_bits(0x7fc0_0011),
        f32::INFINITY,
        f32::NEG_INFINITY,
    ] {
        assert!(!session.set_playback(false, false, invalid).unwrap());
        assert_eq!(session.timeline_foundation().unwrap(), before);
        assert_eq!(source_state(&session), source);
    }
}
