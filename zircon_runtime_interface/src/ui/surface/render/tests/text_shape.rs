use super::sanitized_advance;

#[test]
fn sanitized_advance_rejects_negative_and_non_finite_geometry() {
    assert_eq!(sanitized_advance(12.5), 12.5);
    assert_eq!(sanitized_advance(-4.0), 0.0);
    assert_eq!(sanitized_advance(f32::NAN), 0.0);
    assert_eq!(sanitized_advance(f32::INFINITY), 0.0);
    assert_eq!(sanitized_advance(f32::NEG_INFINITY), 0.0);
}
