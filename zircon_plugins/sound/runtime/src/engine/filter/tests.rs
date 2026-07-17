use zircon_runtime::core::framework::sound::{SoundFilterEffect, SoundFilterMode};

use super::coefficients::SoundBiquadCoefficients;
use super::{apply_biquad_filter_block, SoundBiquadFilterState};

const SAMPLE_RATE_HZ: u32 = 48_000;

fn filter(mode: SoundFilterMode, cutoff_hz: f32) -> SoundFilterEffect {
    SoundFilterEffect {
        mode,
        cutoff_hz,
        resonance: std::f32::consts::FRAC_1_SQRT_2,
        gain_db: 0.0,
    }
}

fn assert_coefficients_are_finite(coefficients: SoundBiquadCoefficients) {
    for coefficient in [
        coefficients.b0,
        coefficients.b1,
        coefficients.b2,
        coefficients.a1,
        coefficients.a2,
    ] {
        assert!(coefficient.is_finite());
    }
}

#[test]
fn every_filter_mode_produces_finite_coefficients() {
    for mode in [
        SoundFilterMode::LowPass,
        SoundFilterMode::HighPass,
        SoundFilterMode::BandPass,
        SoundFilterMode::Notch,
        SoundFilterMode::LowShelf,
        SoundFilterMode::HighShelf,
    ] {
        assert_coefficients_are_finite(SoundBiquadCoefficients::from_filter(
            filter(mode, 1_000.0),
            SAMPLE_RATE_HZ,
        ));
    }
}

#[test]
fn non_finite_filter_parameters_are_normalized_to_finite_output() {
    let mut block = [1.0, -1.0, 0.5, -0.5];
    apply_biquad_filter_block(
        &mut block,
        1,
        0,
        SoundFilterEffect {
            mode: SoundFilterMode::HighShelf,
            cutoff_hz: f32::NAN,
            resonance: f32::NEG_INFINITY,
            gain_db: f32::INFINITY,
        },
        &mut SoundBiquadFilterState::default(),
    );

    assert!(block.into_iter().all(f32::is_finite));
}

#[test]
fn zero_gain_shelves_are_neutral() {
    for mode in [SoundFilterMode::LowShelf, SoundFilterMode::HighShelf] {
        let mut block = [1.0, -0.5, 0.25, -0.125];
        let expected = block;
        apply_biquad_filter_block(
            &mut block,
            1,
            SAMPLE_RATE_HZ,
            filter(mode, 2_000.0),
            &mut SoundBiquadFilterState::default(),
        );

        for (actual, expected) in block.into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1.0e-5);
        }
    }
}

#[test]
fn low_shelf_applies_nonzero_boost_and_cut_gain() {
    let render = |gain_db| {
        let mut block = [1.0; 512];
        let mut effect = filter(SoundFilterMode::LowShelf, 2_000.0);
        effect.gain_db = gain_db;
        apply_biquad_filter_block(
            &mut block,
            1,
            SAMPLE_RATE_HZ,
            effect,
            &mut SoundBiquadFilterState::default(),
        );
        block[384..].iter().sum::<f32>() / 128.0
    };

    assert!(render(6.0) > 1.5);
    assert!(render(-6.0) < 0.75);
}

#[test]
fn low_pass_attenuates_an_alternating_high_frequency_signal() {
    let mut block = [0.0; 256];
    for (index, sample) in block.iter_mut().enumerate() {
        *sample = if index % 2 == 0 { 1.0 } else { -1.0 };
    }
    apply_biquad_filter_block(
        &mut block,
        1,
        SAMPLE_RATE_HZ,
        filter(SoundFilterMode::LowPass, 1_000.0),
        &mut SoundBiquadFilterState::default(),
    );

    assert!(block[128..].iter().all(|sample| sample.abs() < 0.01));
}

#[test]
fn high_pass_rejects_dc_after_its_transient() {
    let mut block = [1.0; 512];
    apply_biquad_filter_block(
        &mut block,
        1,
        SAMPLE_RATE_HZ,
        filter(SoundFilterMode::HighPass, 1_000.0),
        &mut SoundBiquadFilterState::default(),
    );

    assert!(block[384..].iter().all(|sample| sample.abs() < 0.001));
}

#[test]
fn stereo_filter_history_is_channel_isolated() {
    let mut block = [0.0; 32];
    block[0] = 1.0;
    apply_biquad_filter_block(
        &mut block,
        2,
        SAMPLE_RATE_HZ,
        filter(SoundFilterMode::LowPass, 2_000.0),
        &mut SoundBiquadFilterState::default(),
    );

    assert!(block.iter().skip(1).step_by(2).all(|sample| *sample == 0.0));
    assert!(block.iter().step_by(2).any(|sample| *sample != 0.0));
}

#[test]
fn filter_history_continues_across_block_boundaries() {
    let effect = filter(SoundFilterMode::LowPass, 2_000.0);
    let mut state = SoundBiquadFilterState::default();
    let mut first = [0.0; 32];
    first[0] = 1.0;
    let mut second = [0.0; 32];
    apply_biquad_filter_block(&mut first, 1, SAMPLE_RATE_HZ, effect, &mut state);
    apply_biquad_filter_block(&mut second, 1, SAMPLE_RATE_HZ, effect, &mut state);

    let mut combined = [0.0; 64];
    combined[0] = 1.0;
    apply_biquad_filter_block(
        &mut combined,
        1,
        SAMPLE_RATE_HZ,
        effect,
        &mut SoundBiquadFilterState::default(),
    );

    for (split, contiguous) in first.into_iter().chain(second).zip(combined) {
        assert!((split - contiguous).abs() < 1.0e-6);
    }
    assert!(second.into_iter().any(|sample| sample != 0.0));
}

#[test]
fn changing_channel_count_resets_incompatible_history() {
    let mut state = SoundBiquadFilterState::default();
    let effect = filter(SoundFilterMode::LowPass, 2_000.0);
    apply_biquad_filter_block(&mut [1.0], 1, SAMPLE_RATE_HZ, effect, &mut state);

    let mut stereo_silence = [0.0; 8];
    apply_biquad_filter_block(&mut stereo_silence, 2, SAMPLE_RATE_HZ, effect, &mut state);

    assert_eq!(stereo_silence, [0.0; 8]);
}

#[test]
fn zero_channel_filter_is_a_no_op() {
    let mut block = [1.0, -1.0];
    apply_biquad_filter_block(
        &mut block,
        0,
        SAMPLE_RATE_HZ,
        filter(SoundFilterMode::LowPass, 1_000.0),
        &mut SoundBiquadFilterState::default(),
    );

    assert_eq!(block, [1.0, -1.0]);
}
