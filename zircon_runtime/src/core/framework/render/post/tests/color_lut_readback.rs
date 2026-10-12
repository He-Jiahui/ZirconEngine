use super::{RenderColorLutReadbackReference, RenderColorLutReadbackReport};

#[test]
fn color_lut_readback_report_accepts_identity_rgba16float_bytes() {
    let bytes = identity_2x2x2_rgba16float_bytes();

    let report =
        RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes([2, 2, 2], &bytes);

    assert!(report.available);
    assert_eq!(report.byte_len, 64);
    assert_eq!(report.expected_byte_len, 64);
    assert_eq!(report.sample_count, 8);
    assert_eq!(report.reference, RenderColorLutReadbackReference::Identity);
    assert_eq!(report.max_abs_error_micro, 0);
    assert_eq!(report.out_of_tolerance_sample_count, 0);
    assert_eq!(report.identity_out_of_tolerance_sample_count, 0);
    assert_eq!(report.alpha_out_of_tolerance_sample_count, 0);
    assert!(report.reference_within_epsilon());
    assert!(report.identity_within_epsilon());
    assert!(!report.user_lut_within_epsilon());
}

#[test]
fn color_lut_readback_report_rejects_non_identity_rgba16float_bytes() {
    let mut bytes = identity_2x2x2_rgba16float_bytes();
    bytes[8] = 0;
    bytes[9] = 0;

    let report =
        RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes([2, 2, 2], &bytes);

    assert_eq!(report.identity_out_of_tolerance_sample_count, 1);
    assert_eq!(report.out_of_tolerance_sample_count, 1);
    assert!(!report.identity_within_epsilon());
    assert!(!report.reference_within_epsilon());
}

#[test]
fn color_lut_readback_report_tracks_invalid_byte_length() {
    let report =
        RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes([2, 2, 2], &[0; 8]);

    assert!(report.invalid_byte_len);
    assert!(!report.identity_within_epsilon());
    assert!(!report.reference_within_epsilon());
}

#[test]
fn color_lut_readback_report_accepts_user_lut_reference_rgba16float_bytes() {
    let bytes = user_lut_2x2x2_rgba16float_bytes();

    let report = RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
        [2, 2, 2],
        &bytes,
        expected_user_lut_color,
    );

    assert_eq!(report.reference, RenderColorLutReadbackReference::UserLut);
    assert_eq!(report.max_abs_error_micro, 0);
    assert_eq!(report.out_of_tolerance_sample_count, 0);
    assert!(report.reference_within_epsilon());
    assert!(report.user_lut_within_epsilon());
    assert!(!report.identity_within_epsilon());
    assert!(report.identity_out_of_tolerance_sample_count > 0);
}

#[test]
fn color_lut_readback_report_accepts_color_transform_reference_rgba16float_bytes() {
    let bytes = user_lut_2x2x2_rgba16float_bytes();

    let report = RenderColorLutReadbackReport::from_raw_rgba16_float_color_transform_bytes(
        [2, 2, 2],
        &bytes,
        expected_user_lut_color,
    );

    assert_eq!(
        report.reference,
        RenderColorLutReadbackReference::ColorTransform
    );
    assert_eq!(report.reference.diagnostic_id(), 2);
    assert_eq!(report.max_abs_error_micro, 0);
    assert_eq!(report.out_of_tolerance_sample_count, 0);
    assert!(report.reference_within_epsilon());
    assert!(report.color_transform_within_epsilon());
    assert!(!report.user_lut_within_epsilon());
    assert!(!report.identity_within_epsilon());
    assert!(report.identity_out_of_tolerance_sample_count > 0);
}

#[test]
fn color_lut_readback_report_rejects_wrong_user_lut_reference_bytes() {
    let mut bytes = user_lut_2x2x2_rgba16float_bytes();
    bytes[0] = 0;
    bytes[1] = 0;

    let report = RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
        [2, 2, 2],
        &bytes,
        expected_user_lut_color,
    );

    assert_eq!(report.reference, RenderColorLutReadbackReference::UserLut);
    assert_eq!(report.out_of_tolerance_sample_count, 1);
    assert!(!report.reference_within_epsilon());
    assert!(!report.user_lut_within_epsilon());
}

#[test]
fn optimization_batch_20260830ct_color_lut_fuses_reference_and_identity_rgb_scans() {
    let source = include_str!("../color_lut_readback.rs");
    let rgb_scan = ["for channel in ", "0..3 {"].concat();

    assert_eq!(
        source.matches(&rgb_scan).count(),
        1,
        "reference and identity error tracking should share one RGB channel scan"
    );
}

#[test]
#[ignore = "deterministic operation-count benchmark"]
fn optimization_batch_20260830ct_color_lut_fused_rgb_scan_benchmark() {
    const SAMPLE_COUNT: usize = 32_768;
    const CHANNEL_COUNT: usize = 3;

    let mut bytes = vec![0_u8; SAMPLE_COUNT * 8];
    for texel in bytes.chunks_exact_mut(8) {
        texel[6..8].copy_from_slice(&0x3c00_u16.to_le_bytes());
    }
    let report = RenderColorLutReadbackReport::from_raw_rgba16_float_color_transform_bytes(
        [1, 1, SAMPLE_COUNT as u32],
        &bytes,
        |_| [0.0; 3],
    );
    assert_eq!(report.sample_count, SAMPLE_COUNT);
    assert_eq!(report.out_of_tolerance_sample_count, 0);
    assert!(report.identity_out_of_tolerance_sample_count > 0);

    let legacy_channel_iterations = SAMPLE_COUNT * CHANNEL_COUNT * 2;
    let optimized_channel_iterations = SAMPLE_COUNT * CHANNEL_COUNT;
    assert_eq!(optimized_channel_iterations * 2, legacy_channel_iterations);
    println!(
        "RUNTIME507_COLOR_LUT_FUSED_RGB_SCAN_BENCH_V1 samples={SAMPLE_COUNT} \
             legacy_channel_iterations={legacy_channel_iterations} \
             optimized_channel_iterations={optimized_channel_iterations} reduction_percent=50"
    );
}

fn identity_2x2x2_rgba16float_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    for z in [0_u16, 0x3c00] {
        for y in [0_u16, 0x3c00] {
            for x in [0_u16, 0x3c00] {
                push_f16(&mut bytes, x);
                push_f16(&mut bytes, y);
                push_f16(&mut bytes, z);
                push_f16(&mut bytes, 0x3c00);
            }
        }
    }
    bytes
}

fn user_lut_2x2x2_rgba16float_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    for z in [0.0_f32, 1.0] {
        for y in [0.0_f32, 1.0] {
            for x in [0.0_f32, 1.0] {
                let expected = expected_user_lut_color([x, y, z]);
                push_f16(&mut bytes, f16_bits_for_test_value(expected[0]));
                push_f16(&mut bytes, f16_bits_for_test_value(expected[1]));
                push_f16(&mut bytes, f16_bits_for_test_value(expected[2]));
                push_f16(&mut bytes, 0x3c00);
            }
        }
    }
    bytes
}

fn expected_user_lut_color(source_color: [f32; 3]) -> [f32; 3] {
    [
        1.0 - source_color[0],
        source_color[1] * 0.5,
        source_color[2],
    ]
}

fn f16_bits_for_test_value(value: f32) -> u16 {
    match value {
        0.0 => 0,
        0.5 => 0x3800,
        1.0 => 0x3c00,
        other => panic!("unsupported test half-float value {other}"),
    }
}

fn push_f16(bytes: &mut Vec<u8>, bits: u16) {
    bytes.extend_from_slice(&bits.to_le_bytes());
}
