use super::super::super::super::super::*;

#[test]
fn production_dsp_rejects_non_finite_delay_feedback_before_render() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 1,
                feedback: f32::INFINITY,
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("feedback"));
}
