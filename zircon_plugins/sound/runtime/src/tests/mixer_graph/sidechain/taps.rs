use super::super::super::*;
use super::super::support::render_sidechain_tap_mix;

#[test]
fn sidechain_compressor_respects_pre_and_post_effect_taps() {
    let pre_effect_mix = render_sidechain_tap_mix(true);
    let post_effect_mix = render_sidechain_tap_mix(false);

    assert!(pre_effect_mix[0] < 0.5);
    assert_sample_near(post_effect_mix[0], 0.5);
}
