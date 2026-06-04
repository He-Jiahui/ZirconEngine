use super::super::super::*;

#[test]
fn looping_timeline_sequence_wraps_and_keeps_sequence_alive() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("timeline.loop.cutoff");
    let binding = SoundAutomationBindingId::new(105);

    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Timeline/Loop:sound.timeline.cutoff".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    sound
        .schedule_timeline_sequence(SoundTimelineSequence::new(
            SoundTimelineSequenceId::new("looping-cutoff"),
            1.0,
            true,
            vec![SoundTimelineAutomationTrack {
                binding,
                curve: SoundAutomationCurve::from_keyframes([
                    SoundAutomationKeyframe::linear(0.0, 0.0),
                    SoundAutomationKeyframe::linear(1.0, 1.0),
                ]),
            }],
        ))
        .unwrap();

    let report = sound.advance_timeline_sequences(1.25).unwrap();

    assert_eq!(report.len(), 1);
    assert!(!report[0].completed);
    assert_sample_near(report[0].time_seconds, 0.25);
    assert_sample_near(sound.parameter_value(&parameter).unwrap(), 0.25);
    assert_eq!(sound.timeline_sequences().unwrap().len(), 1);
}
