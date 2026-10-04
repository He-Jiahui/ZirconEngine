use super::*;

#[test]
fn rectangle_query_includes_intersecting_candidate_bounds() {
    assert!(circle_intersects_rect(
        Vec2::new(12.0, 10.0),
        3.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
    assert!(!circle_intersects_rect(
        Vec2::new(14.0, 10.0),
        3.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
}

#[test]
fn segment_query_detects_crossing_and_rejects_separated_segments() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    assert!(segment_intersects_rect(
        Vec2::new(-5.0, 5.0),
        Vec2::new(15.0, 5.0),
        min,
        max,
    ));
    assert!(!segment_intersects_rect(
        Vec2::new(-5.0, 15.0),
        Vec2::new(15.0, 15.0),
        min,
        max,
    ));
}
