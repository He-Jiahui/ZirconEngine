use super::geometry::clamp_to_ordered_range;

#[test]
fn rounded_rect_center_clamp_tolerates_crossed_float_bounds() {
    let clamped = clamp_to_ordered_range(40.0, 40.0, 39.999_992);
    assert!((clamped - 39.999_996).abs() <= f32::EPSILON);
}
