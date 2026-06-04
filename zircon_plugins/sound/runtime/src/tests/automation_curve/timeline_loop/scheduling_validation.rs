use super::super::super::*;

#[test]
fn timeline_loop_scheduling_validation_is_typed() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .schedule_timeline_sequence(SoundTimelineSequence::new(
            SoundTimelineSequenceId::new("bad-empty"),
            1.0,
            false,
            Vec::new(),
        ))
        .unwrap_err()
        .to_string()
        .contains("at least one automation track"));
    assert!(matches!(
        sound
            .schedule_timeline_sequence(SoundTimelineSequence::new(
                SoundTimelineSequenceId::new("bad-binding"),
                1.0,
                false,
                vec![SoundTimelineAutomationTrack {
                    binding: SoundAutomationBindingId::new(9999),
                    curve: SoundAutomationCurve::from_keyframes([
                        SoundAutomationKeyframe::linear(0.0, 0.0),
                        SoundAutomationKeyframe::linear(1.0, 1.0),
                    ]),
                }],
            ))
            .unwrap_err(),
        SoundError::UnknownAutomationBinding { .. }
    ));
}
