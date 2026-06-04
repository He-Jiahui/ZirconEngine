use super::super::super::*;
use super::super::support::render_master_effect;

#[test]
fn filter_effect_is_deterministic() {
    let low_pass = render_master_effect(
        test_effect(SoundEffectKind::Filter(SoundFilterEffect {
            mode: SoundFilterMode::LowPass,
            cutoff_hz: 1_000.0,
            resonance: 0.0,
            gain_db: 0.0,
        })),
        &[1.0, 0.0],
        2,
    );
    assert!(low_pass[0] > 0.0 && low_pass[0] < 0.2);
    assert!(low_pass[2] > low_pass[0] && low_pass[2] < 0.1);
}
