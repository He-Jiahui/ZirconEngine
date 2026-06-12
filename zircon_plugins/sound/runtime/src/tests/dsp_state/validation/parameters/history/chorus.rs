use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_unbounded_chorus_history_before_render() {
    let sound = DefaultSoundManager::default();
    let oversized_history = 1_000_000;

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Chorus(SoundChorusEffect {
                voices: 2,
                delay_frames: oversized_history,
                depth_frames: 1,
                rate_hz: 0.25,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("history budget"));
}
