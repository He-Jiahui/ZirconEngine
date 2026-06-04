use super::super::*;

#[test]
fn mixer_snapshot_reports_effect_and_track_delay_latency() {
    let sound = DefaultSoundManager::default();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 7,
                feedback: 0.0,
            })),
        )
        .unwrap();

    sound.render_mix(1).unwrap();
    assert_eq!(sound.mixer_snapshot().unwrap().latency_frames, 7);

    let mut master = SoundTrackDescriptor::master();
    master.controls.delay_frames = 11;
    sound.add_or_update_track(master).unwrap();
    sound.render_mix(1).unwrap();
    assert_eq!(sound.mixer_snapshot().unwrap().latency_frames, 11);
}
