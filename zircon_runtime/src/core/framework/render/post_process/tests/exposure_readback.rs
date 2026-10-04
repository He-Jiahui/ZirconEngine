use super::RenderExposureReadbackReport;

#[test]
fn exposure_readback_report_accepts_valid_history_words() {
    let report = RenderExposureReadbackReport::from_words([1.25, 9.5, 9.25, 1.0]);

    assert!(report.available);
    assert_eq!(report.byte_len, 16);
    assert_eq!(report.expected_byte_len, 16);
    assert!(!report.invalid_byte_len);
    assert_eq!(report.invalid_word_count, 0);
    assert_eq!(report.multiplier(), 1.25);
    assert_eq!(report.resolved_ev100(), 9.5);
    assert_eq!(report.average_ev100(), 9.25);
    assert_eq!(report.valid_flag(), 1.0);
    assert_eq!(report.multiplier_micro(), 1_250_000);
    assert_eq!(report.valid_flag_micro(), 1_000_000);
    assert!(report.history_valid());
    assert!(report.multiplier_within_epsilon(1.25, 0.0001));
}

#[test]
fn exposure_readback_report_rejects_invalid_length_and_nan_words() {
    let mut bytes = [0_u8; 12];
    bytes[0..4].copy_from_slice(&f32::NAN.to_le_bytes());

    let report = RenderExposureReadbackReport::from_raw_f32x4_bytes(&bytes);

    assert!(report.available);
    assert!(report.invalid_byte_len);
    assert_eq!(report.invalid_word_count, 1);
    assert!(!report.history_valid());
}
