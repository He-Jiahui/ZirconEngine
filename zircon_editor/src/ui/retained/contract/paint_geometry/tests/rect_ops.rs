use super::*;

#[test]
fn corner_radius_stays_within_a_narrow_frame() {
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.5,
        height: 24.0,
    };

    assert_eq!(corner_radius_for_frame(&frame, 4.0), 0.25);
}

#[test]
fn bounded_extent_rejects_negative_and_non_finite_values() {
    assert_eq!(bounded_extent(12.5), 12.5);
    assert_eq!(bounded_extent(-1.0), 0.0);
    assert_eq!(bounded_extent(f32::NAN), 0.0);
    assert_eq!(bounded_extent(f32::INFINITY), 0.0);
}

#[test]
fn intersection_requires_more_than_half_a_pixel_of_coverage() {
    let left = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    };

    assert!(intersect(
        &left,
        &FrameRect {
            x: 9.5,
            ..left.clone()
        }
    )
    .is_none());
    assert!(intersect(&left, &FrameRect { x: 9.49, ..left }).is_some());
}

#[test]
fn inward_pixel_alignment_preserves_fractional_frame_containment() {
    let frame = FrameRect {
        x: 8.4,
        y: 6.6,
        width: 80.4,
        height: 40.4,
    };

    let aligned = inward_pixel_aligned_rect(&frame);

    assert!(aligned.x >= frame.x);
    assert!(aligned.y >= frame.y);
    assert!(aligned.right() <= frame.right());
    assert!(aligned.bottom() <= frame.bottom());
}
