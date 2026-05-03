use super::*;

#[test]
fn automation_curve_sample_updates_bound_synth_parameter() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("synth.curve_amp");
    let binding = SoundAutomationBindingId::new(101);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Synth:sound.synth.curve_amp".to_string(),
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

    let curve = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::linear(0.0, 0.2),
        SoundAutomationKeyframe::smooth_step(1.0, 0.8),
        SoundAutomationKeyframe::linear(2.0, 0.4),
    ]);

    let value = sound
        .apply_automation_curve_sample(binding, &curve, 0.5)
        .unwrap();

    assert_sample_near(value, 0.5);
    assert_sample_near(sound.parameter_value(&parameter).unwrap(), 0.5);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}

#[test]
fn automation_curve_supports_step_and_endpoint_clamping() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("synth.stepped");
    let binding = SoundAutomationBindingId::new(102);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Synth:sound.synth.stepped".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    let curve = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::step(1.0, 0.25),
        SoundAutomationKeyframe::linear(2.0, 0.75),
    ]);

    assert_sample_near(
        sound
            .apply_automation_curve_sample(binding, &curve, 0.0)
            .unwrap(),
        0.25,
    );
    assert_sample_near(
        sound
            .apply_automation_curve_sample(binding, &curve, 1.5)
            .unwrap(),
        0.25,
    );
    assert_sample_near(
        sound
            .apply_automation_curve_sample(binding, &curve, 3.0)
            .unwrap(),
        0.75,
    );
}

#[test]
fn automation_curve_rejects_invalid_curve_data_cleanly() {
    let sound = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(103);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Master:sound.master.gain".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();

    let empty = SoundAutomationCurve::from_keyframes(Vec::<SoundAutomationKeyframe>::new());
    assert!(sound
        .apply_automation_curve_sample(binding, &empty, 0.0)
        .unwrap_err()
        .to_string()
        .contains("at least one keyframe"));

    let unsorted = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::linear(1.0, 0.5),
        SoundAutomationKeyframe::linear(1.0, 0.75),
    ]);
    assert!(sound
        .apply_automation_curve_sample(binding, &unsorted, 0.0)
        .unwrap_err()
        .to_string()
        .contains("strictly increasing"));

    let non_finite =
        SoundAutomationCurve::from_keyframes([SoundAutomationKeyframe::linear(0.0, f32::NAN)]);
    assert!(sound
        .apply_automation_curve_sample(binding, &non_finite, 0.0)
        .unwrap_err()
        .to_string()
        .contains("finite"));
}

#[test]
fn timeline_sequence_advances_sound_automation_and_completes() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("timeline.music.intensity");
    let binding = SoundAutomationBindingId::new(104);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Timeline/Music:intensity".to_string(),
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

#[test]
fn looping_timeline_sequence_wraps_and_validation_is_typed() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("timeline.loop.cutoff");
    let binding = SoundAutomationBindingId::new(105);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Timeline/Loop:cutoff".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();

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
