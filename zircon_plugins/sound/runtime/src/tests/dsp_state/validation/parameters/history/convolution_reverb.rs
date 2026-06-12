use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_unbounded_convolution_reverb_history_before_render() {
    let sound = DefaultSoundManager::default();
    let oversized_history = 1_000_000;

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::ConvolutionReverb(
                SoundConvolutionReverbEffect {
                    impulse_response: SoundImpulseResponseId::new(44),
                    fallback_to_algorithmic: true,
                    latency_frames: oversized_history,
                },
            )),
        )
        .unwrap_err()
        .to_string()
        .contains("history budget"));
}
