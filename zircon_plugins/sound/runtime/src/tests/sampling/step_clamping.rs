// 经停机合成参数绑定观察阶跃采样与端点钳制，核对三处具体采样值。
use super::super::super::*;

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
fn timeline_step_gain_applies_exact_terminal_key() {
    let sound = step_gain_timeline(false);
    assert_step_gain_advance(&sound, 0.75, 0.75, false, 0.5);
    assert_step_gain_advance(&sound, 0.25, 1.0, true, 0.25);
}

#[test]
fn timeline_step_gain_applies_terminal_key_when_overshooting() {
    let sound = step_gain_timeline(false);
    assert_step_gain_advance(&sound, 0.75, 0.75, false, 0.5);
    assert_step_gain_advance(&sound, 0.5, 1.0, true, 0.25);
}

#[test]
fn timeline_step_gain_wraps_at_exact_duration_when_looping() {
    let sound = step_gain_timeline(true);
    assert_step_gain_advance(&sound, 0.75, 0.75, false, 0.5);
    assert_step_gain_advance(&sound, 0.25, 0.0, false, 0.75);
}

fn step_gain_timeline(looping: bool) -> DefaultSoundManager {
    let sound = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(106);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Master:sound.master.gain".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();
    sound
        .schedule_timeline_sequence(SoundTimelineSequence::new(
            SoundTimelineSequenceId::new("step-master-gain"),
            1.0,
            looping,
            vec![SoundTimelineAutomationTrack {
                binding,
                curve: SoundAutomationCurve::from_keyframes([
                    SoundAutomationKeyframe::step(0.0, 0.75),
                    SoundAutomationKeyframe::step(0.5, 0.5),
                    SoundAutomationKeyframe::step(1.0, 0.25),
                ]),
            }],
        ))
        .unwrap();
    sound
}

fn assert_step_gain_advance(
    sound: &DefaultSoundManager,
    delta_seconds: f32,
    expected_time: f32,
    completed: bool,
    expected_gain: f32,
) {
    let reports = sound.advance_timeline_sequences(delta_seconds).unwrap();
    assert_eq!(reports.len(), 1);
    let report = &reports[0];
    assert_eq!(report.sequence.as_str(), "step-master-gain");
    assert_eq!(report.time_seconds, expected_time);
    assert_eq!(report.completed, completed);
    assert_eq!(report.samples.len(), 1);
    assert_eq!(
        report.samples[0].binding,
        SoundAutomationBindingId::new(106)
    );
    assert_eq!(sound.timeline_sequences().unwrap().is_empty(), completed);

    let snapshot = sound.mixer_snapshot().unwrap();
    let gain = snapshot.graph.master_track().unwrap().controls.gain;
    assert_eq!(
        (report.samples[0].value, gain),
        (expected_gain, expected_gain)
    );
}
