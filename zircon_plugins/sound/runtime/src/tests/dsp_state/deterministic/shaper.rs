use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn waveshaper_effect_is_deterministic() {
    let shaped = render_master_effect(
        test_effect(SoundEffectKind::WaveShaper(SoundWaveShaperEffect {
            drive: 2.0,
        })),
        &[0.5],
        1,
    );
    assert!(shaped[0] > 0.5 && shaped[0] <= 1.0);
}
