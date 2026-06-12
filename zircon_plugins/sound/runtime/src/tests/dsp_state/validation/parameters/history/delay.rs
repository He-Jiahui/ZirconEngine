use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_unbounded_delay_history_before_render() {
    let sound = DefaultSoundManager::default();
    let oversized_history = 1_000_000;

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: oversized_history,
                feedback: 0.0,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("history budget"));
}
