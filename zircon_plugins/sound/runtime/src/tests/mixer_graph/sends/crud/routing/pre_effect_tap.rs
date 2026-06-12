use super::super::super::super::super::*;

use super::tap_mix::render_track_send_tap_mix;

#[test]
fn track_send_pre_effect_tap_routes_raw_track_signal_to_target_bus() {
    let mix = render_track_send_tap_mix(true);

    assert_samples_near(&mix, &[0.625, 0.625]);
}
