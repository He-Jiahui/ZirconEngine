use super::dynamics::{compressor_block, limit};
use super::gain::{multiply, wet_mix};
use super::history::SoundHistoryState;
use super::meter::meter_for;
use super::modulation::{modulated_delay, phaser_block};
use super::reverb::{convolve_block, reverb_block};
use super::shaper::waveshape;
use super::stereo::pan_stereo;
use zircon_runtime::core::framework::sound::SoundTrackId;

#[test]
fn modulation_history_golden_is_preserved_for_m2() {
    let mut history = SoundHistoryState::default();
    let mut phase = 0.0;
    let mut first = [0.5];
    modulated_delay(
        &mut first,
        1,
        48_000,
        1,
        0,
        0.0,
        0.0,
        &mut history,
        &mut phase,
    );
    let mut second = [0.0];
    modulated_delay(
        &mut second,
        1,
        48_000,
        1,
        0,
        0.0,
        0.0,
        &mut history,
        &mut phase,
    );
    assert_eq!(first, [0.5]);
    assert_eq!(second, [0.25]);
}

#[test]
fn phaser_phase_golden_is_preserved_for_m2() {
    let mut phase = 0.0;
    let mut first = [1.0];
    phaser_block(&mut first, 1, 48_000, 12_000.0, 1.0, 0.25, &mut phase);
    let mut second = [1.0];
    phaser_block(&mut second, 1, 48_000, 12_000.0, 1.0, 0.25, &mut phase);
    assert!(first[0].abs() < 1.0e-6);
    assert!((second[0] - 0.5).abs() < 1.0e-6);
}

#[test]
fn convolution_tail_golden_is_preserved_for_m2() {
    let mut history = SoundHistoryState::default();
    let mut first = [1.0];
    convolve_block(&mut first, 1, &[0.5, 0.25], &mut history);
    let mut second = [0.0];
    convolve_block(&mut second, 1, &[0.5, 0.25], &mut history);
    assert_eq!(first, [0.5]);
    assert_eq!(second, [0.25]);
}

#[test]
fn limiter_golden_is_preserved_for_m2() {
    let mut buffer = [2.0, -2.0, 0.5];
    limit(&mut buffer, 1.0);
    assert_eq!(buffer, [1.0, -1.0, 0.5]);
}

#[test]
fn compressor_zero_attack_applies_expected_ratio_and_makeup() {
    let mut buffer = [1.0, -1.0];
    let mut gain_state = 1.0;
    compressor_block(
        &mut buffer,
        2,
        48_000,
        -6.020_6,
        2.0,
        0.0,
        0.0,
        6.020_6,
        None,
        &mut gain_state,
    );
    assert!((buffer[0] - 2.0_f32.sqrt()).abs() < 1.0e-4);
    assert!((buffer[1] + 2.0_f32.sqrt()).abs() < 1.0e-4);
}

#[test]
fn compressor_sidechain_drives_gain_without_replacing_program_audio() {
    let mut buffer = [0.25];
    let sidechain = [1.0];
    let mut gain_state = 1.0;
    compressor_block(
        &mut buffer,
        1,
        48_000,
        -6.020_6,
        4.0,
        0.0,
        0.0,
        0.0,
        Some(&sidechain),
        &mut gain_state,
    );
    assert!(buffer[0] < 0.25);
    assert!(buffer[0] > 0.0);
}

#[test]
fn gain_and_wet_mix_preserve_dry_wet_contract() {
    let dry = [1.0, -1.0];
    let mut wet = [0.5, 0.5];
    multiply(&mut wet, 2.0);
    wet_mix(&mut wet, &dry, 0.25);
    assert_eq!(wet, [1.0, -0.5]);
}

#[test]
fn mono_meter_projects_peak_and_rms_to_both_channels() {
    let meter = meter_for(SoundTrackId::new(7), &[1.0, -1.0, 0.0, 0.0], 1);
    assert_eq!(meter.peak_left, 1.0);
    assert_eq!(meter.peak_right, 1.0);
    assert!((meter.rms_left - 2.0_f32.sqrt() * 0.5).abs() < 1.0e-6);
    assert_eq!(meter.rms_left, meter.rms_right);
}

#[test]
fn stereo_meter_keeps_channel_peaks_and_rms_independent() {
    let meter = meter_for(SoundTrackId::new(8), &[1.0, 0.5, -1.0, -0.5], 2);
    assert_eq!(meter.peak_left, 1.0);
    assert_eq!(meter.peak_right, 0.5);
    assert_eq!(meter.rms_left, 1.0);
    assert_eq!(meter.rms_right, 0.5);
}

#[test]
fn waveshaper_is_odd_bounded_and_preserves_zero() {
    let mut buffer = [-1.0, 0.0, 1.0];
    waveshape(&mut buffer, 4.0);
    assert!((buffer[0] + 1.0).abs() < 1.0e-6);
    assert_eq!(buffer[1], 0.0);
    assert!((buffer[2] - 1.0).abs() < 1.0e-6);
}

#[test]
fn stereo_pan_width_gain_and_phase_are_applied_per_channel() {
    let mut buffer = [1.0, -1.0];
    pan_stereo(&mut buffer, 2, 0.5, 1.0, 0.5, 2.0, true, false);
    assert!((buffer[0] + 0.25).abs() < 1.0e-6);
    assert!((buffer[1] + 2.0).abs() < 1.0e-6);
}

#[test]
fn algorithmic_reverb_emits_damped_tail_from_history() {
    let mut history = SoundHistoryState::default();
    let mut impulse = [1.0, 0.0, 0.0];
    reverb_block(&mut impulse, 1, 1, 3, 0.5, &mut history);
    let mut tail = [0.0, 0.0, 0.0];
    reverb_block(&mut tail, 1, 1, 3, 0.5, &mut history);
    assert!(tail.iter().any(|sample| sample.abs() > 0.0));
}

#[test]
fn convolution_identity_impulse_preserves_stereo_samples() {
    let mut history = SoundHistoryState::default();
    let mut buffer = [0.25, -0.5, 0.75, -1.0];
    convolve_block(&mut buffer, 2, &[1.0], &mut history);
    assert_eq!(buffer, [0.25, -0.5, 0.75, -1.0]);
}

#[test]
fn modulation_history_remains_channel_isolated() {
    let mut history = SoundHistoryState::default();
    let mut phase = 0.0;
    let mut first = [1.0, -1.0];
    modulated_delay(
        &mut first,
        2,
        48_000,
        1,
        0,
        0.0,
        0.0,
        &mut history,
        &mut phase,
    );
    let mut second = [0.0, 0.0];
    modulated_delay(
        &mut second,
        2,
        48_000,
        1,
        0,
        0.0,
        0.0,
        &mut history,
        &mut phase,
    );
    assert_eq!(second, [0.5, -0.5]);
}

#[test]
fn history_keeps_only_the_requested_tail_frames() {
    let mut history = SoundHistoryState::default();
    history.remember(&[1.0, 2.0, 3.0, 4.0], 2, 1);

    assert_eq!(history.sample(&[5.0], 1, 0, 0, 1), 4.0);
    assert_eq!(history.sample(&[5.0], 1, 0, 0, 2), 3.0);
}

#[test]
fn zero_history_capacity_clears_retained_samples() {
    let mut history = SoundHistoryState::default();
    history.remember(&[1.0], 1, 1);
    history.remember(&[], 0, 1);

    assert_eq!(history.sample(&[0.0], 1, 0, 0, 1), 0.0);
}

#[test]
fn compressor_recovers_a_non_finite_gain_state() {
    let mut buffer = [0.1];
    let mut gain_state = f32::NAN;
    compressor_block(
        &mut buffer,
        1,
        48_000,
        0.0,
        4.0,
        0.0,
        0.0,
        0.0,
        None,
        &mut gain_state,
    );

    assert_eq!(buffer, [0.1]);
    assert_eq!(gain_state, 1.0);
}

#[test]
fn negative_limiter_ceiling_mutes_the_block() {
    let mut buffer = [1.0, -1.0];
    limit(&mut buffer, -1.0);
    assert_eq!(buffer, [0.0, 0.0]);
}

#[test]
fn modulation_feedback_is_clamped_to_the_stable_upper_bound() {
    let mut history = SoundHistoryState::default();
    history.remember(&[1.0], 1, 1);
    let mut phase = 0.0;
    let mut block = [0.0];
    modulated_delay(
        &mut block,
        1,
        48_000,
        1,
        0,
        0.0,
        10.0,
        &mut history,
        &mut phase,
    );

    assert!((block[0] - 0.975).abs() < 1.0e-6);
}

#[test]
fn reverb_damping_is_clamped_below_unity() {
    let render = |damping| {
        let mut history = SoundHistoryState::default();
        let mut impulse = [1.0, 0.0, 0.0];
        reverb_block(&mut impulse, 1, 1, 3, damping, &mut history);
        let mut tail = [0.0, 0.0, 0.0];
        reverb_block(&mut tail, 1, 1, 3, damping, &mut history);
        tail
    };

    assert_eq!(render(1.0), render(0.99));
}

#[test]
fn empty_convolution_impulse_is_a_no_op() {
    let mut history = SoundHistoryState::default();
    let mut buffer = [0.25, -0.5];
    convolve_block(&mut buffer, 2, &[], &mut history);
    assert_eq!(buffer, [0.25, -0.5]);
}

#[test]
fn mono_pan_uses_the_single_channel_without_index_aliasing() {
    let mut buffer = [1.0];
    pan_stereo(&mut buffer, 1, 0.5, 1.0, 1.0, 1.0, false, false);
    assert_eq!(buffer, [0.5]);
}

#[test]
fn zero_channel_pan_leaves_the_buffer_unchanged() {
    let mut buffer = [1.0, -1.0];
    pan_stereo(&mut buffer, 0, 0.5, 1.0, 1.0, 1.0, false, false);
    assert_eq!(buffer, [1.0, -1.0]);
}

#[test]
fn empty_meter_is_silent() {
    let track = SoundTrackId::new(9);
    let meter = meter_for(track, &[], 2);
    assert_eq!(meter.track, track);
    assert_eq!(meter.peak_left, 0.0);
    assert_eq!(meter.peak_right, 0.0);
    assert_eq!(meter.rms_left, 0.0);
    assert_eq!(meter.rms_right, 0.0);
}

#[test]
fn full_wet_mix_does_not_read_or_apply_the_dry_buffer() {
    let mut wet = [0.25, -0.5];
    wet_mix(&mut wet, &[1.0], 1.0);
    assert_eq!(wet, [0.25, -0.5]);
}

#[test]
fn negative_waveshaper_drive_uses_the_neutral_curve() {
    let mut buffer = [-1.0, 0.0, 1.0];
    waveshape(&mut buffer, -10.0);
    assert!((buffer[0] + 1.0).abs() < 1.0e-6);
    assert_eq!(buffer[1], 0.0);
    assert!((buffer[2] - 1.0).abs() < 1.0e-6);
}
