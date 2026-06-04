use super::super::*;

#[test]
fn timeline_sequence_advances_sound_automation_and_completes() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("timeline.music.intensity");
    let binding = SoundAutomationBindingId::new(104);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Timeline/Music:sound.timeline.intensity".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter: parameter.clone(),
                default_value: 0.0,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let sequence = SoundTimelineSequence::new(
        SoundTimelineSequenceId::new("music-intensity-rise"),
        1.0,
        false,
        vec![SoundTimelineAutomationTrack {
            binding,
            curve: SoundAutomationCurve::from_keyframes([
                SoundAutomationKeyframe::linear(0.0, 0.0),
                SoundAutomationKeyframe::linear(1.0, 1.0),
            ]),
        }],
    );
    sound.schedule_timeline_sequence(sequence).unwrap();

    let first = sound.advance_timeline_sequences(0.25).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].sequence.as_str(), "music-intensity-rise");
    assert!(!first[0].completed);
    assert_sample_near(first[0].samples[0].value, 0.25);
    assert_sample_near(sound.parameter_value(&parameter).unwrap(), 0.25);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.25, 0.25]);

    let second = sound.advance_timeline_sequences(0.75).unwrap();
    assert!(second[0].completed);
    assert_sample_near(second[0].samples[0].value, 1.0);
    assert!(sound.timeline_sequences().unwrap().is_empty());
}
