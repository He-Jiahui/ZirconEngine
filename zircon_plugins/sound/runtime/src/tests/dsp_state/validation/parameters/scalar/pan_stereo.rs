use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_out_of_range_stereo_pan_before_render() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::PanStereo(SoundPanStereoEffect {
                pan: 1.5,
                width: 1.0,
                left_gain: 1.0,
                right_gain: 1.0,
                invert_left_phase: false,
                invert_right_phase: false,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("stereo pan"));
}
