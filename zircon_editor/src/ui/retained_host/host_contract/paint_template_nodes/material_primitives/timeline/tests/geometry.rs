use super::*;

#[test]
fn centered_square_does_not_expand_a_collapsed_timeline_slot() {
    let square = centered_square(&FrameRect {
        x: 12.0,
        y: 8.0,
        width: 0.0,
        height: 24.0,
    });

    assert_eq!(square.width, 0.0);
    assert_eq!(square.height, 0.0);
}

#[test]
fn centered_square_preserves_fractional_analytic_dot_geometry() {
    let square = centered_square(&FrameRect {
        x: 12.25,
        y: 8.5,
        width: 18.5,
        height: 14.75,
    });

    assert_eq!(
        square,
        FrameRect {
            x: 14.125,
            y: 8.5,
            width: 14.75,
            height: 14.75,
        }
    );
}
