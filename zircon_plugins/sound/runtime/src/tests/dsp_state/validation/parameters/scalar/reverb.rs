use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_out_of_range_reverb_room_size_before_render() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 1.25,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: 2,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("room size"));
}
