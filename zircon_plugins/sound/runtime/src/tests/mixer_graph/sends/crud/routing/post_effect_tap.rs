use super::super::super::super::super::*;

use super::tap_mix::render_track_send_tap_mix;

#[test]
fn track_send_post_effect_tap_routes_processed_track_signal_to_target_bus() {
    let mix = render_track_send_tap_mix(false);

    assert_samples_near(&mix, &[0.25, 0.25]);
}
