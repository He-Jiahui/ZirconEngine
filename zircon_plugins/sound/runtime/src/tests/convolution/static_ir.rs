use super::super::*;

#[test]
fn static_convolution_impulse_response_processes_master_track() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/impulse.wav", &[1.0, 0.0]));
    let impulse_response = SoundImpulseResponseId::new(1);
    sound
        .set_impulse_response(impulse_response, vec![0.5, 0.25])
        .unwrap();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            SoundEffectDescriptor::new(
                SoundEffectId::new(3),
                "Static IR",
                SoundEffectKind::ConvolutionReverb(SoundConvolutionReverbEffect {
                    impulse_response,
                    fallback_to_algorithmic: true,
                    latency_frames: 1,
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(2).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5, 0.25, 0.25]);
}
