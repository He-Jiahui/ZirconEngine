use super::*;

#[test]
fn text_msdf_median_decode_matches_msdfgen() {
    let reference_samples = [
        ([0.1, 0.2, 0.3, 1.0], 0.2),
        ([0.3, 0.1, 0.2, 1.0], 0.2),
        ([0.2, 0.3, 0.1, 1.0], 0.2),
        ([0.7, 0.7, 0.9, 1.0], 0.7),
        ([0.49, 0.51, 0.50, 1.0], 0.50),
    ];

    for (sample, expected) in reference_samples {
        assert!((msdf_sample_distance(sample) - expected).abs() < f32::EPSILON);
    }
}

#[test]
fn text_msdf_mtsdf_true_distance_decode_uses_alpha() {
    let sample = [0.1, 0.9, 0.6, 0.42];

    assert!((msdf_sample_distance(sample) - 0.6).abs() < f32::EPSILON);
    assert!((mtsdf_sample_true_distance(sample) - 0.42).abs() < f32::EPSILON);
}

#[test]
fn text_msdf_reference_coverage_clamps_and_keeps_one_pixel_minimum_range() {
    assert_eq!(distance_field_coverage(0.0, 0.0, 0.0), 0.0);
    assert_eq!(distance_field_coverage(0.5, 0.0, 0.0), 0.5);
    assert_eq!(distance_field_coverage(1.0, 0.0, 0.0), 1.0);
    assert!(distance_field_coverage(0.55, 8.0, 0.125) > 0.5);
    assert!(distance_field_coverage(0.45, 8.0, 0.125) < 0.5);
}
