use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_unbounded_reverb_history_before_render() {
    let sound = DefaultSoundManager::default();
    let oversized_history = 1_000_000;

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 0.5,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: oversized_history,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("history budget"));
}
