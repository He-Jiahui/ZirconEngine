use super::super::*;

#[test]
fn clip_and_external_inputs_resample_to_mixer_rate() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/resampled.wav",
        24_000,
        &[0.25, 0.5],
    ));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(
        &sound.render_mix(4).unwrap().samples,
        &[0.25, 0.25, 0.375, 0.375, 0.5, 0.5, 0.5, 0.5],
    );

    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("synth.low-rate");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 24_000,
                channel_count: 1,
                samples: vec![0.5, 1.0],
            },
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    assert_samples_near(
        &sound.render_mix(4).unwrap().samples,
        &[0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0, 1.0],
    );
}
