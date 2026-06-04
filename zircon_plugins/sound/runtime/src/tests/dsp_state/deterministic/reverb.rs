use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn reverb_effect_is_deterministic() {
    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 0.5,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: 2,
            })),
            &[0.5, 0.0],
            2,
        ),
        &[0.5, 0.5, 0.375, 0.375],
    );
}
