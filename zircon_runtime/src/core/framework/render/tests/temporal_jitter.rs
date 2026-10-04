use super::{halton, TemporalJitterSequence};

#[test]
fn render_taa_halton_matches_reference_values() {
    assert_close(halton(1, 2), 0.5);
    assert_close(halton(2, 2), 0.25);
    assert_close(halton(3, 2), 0.75);
    assert_close(halton(1, 3), 1.0 / 3.0);
    assert_close(halton(2, 3), 2.0 / 3.0);
    assert_close(halton(3, 3), 1.0 / 9.0);
}

#[test]
fn render_taa_jitter_sequence_is_periodic_and_avoids_zero_index() {
    let sequence = TemporalJitterSequence::new(8);
    let first = sequence.sample(0);
    let repeated = sequence.sample(8);

    assert_eq!(sequence.period(), 8);
    assert_eq!(first.sequence_index, 1);
    assert_eq!(repeated.sequence_index, 1);
    assert_close(first.offset_pixels.x, 0.0);
    assert_close(first.offset_pixels.y, -1.0 / 6.0);
    assert_eq!(first, repeated);
}

#[test]
fn render_taa_jitter_sequence_clamps_zero_period() {
    let sequence = TemporalJitterSequence::new(0);

    assert_eq!(sequence.period(), 1);
    assert_eq!(sequence.sample(99).sequence_index, 1);
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "expected {actual} to be close to {expected}"
    );
}
