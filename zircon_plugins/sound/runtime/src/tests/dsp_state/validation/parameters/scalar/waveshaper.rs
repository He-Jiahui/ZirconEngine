use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_negative_waveshaper_drive_before_render() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::WaveShaper(SoundWaveShaperEffect {
                drive: -0.1,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("drive"));
}
