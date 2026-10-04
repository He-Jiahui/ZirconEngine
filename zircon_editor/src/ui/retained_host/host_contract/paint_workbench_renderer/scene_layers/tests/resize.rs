use super::*;

#[test]
fn splitter_visual_keeps_large_hit_target_but_draws_one_pixel_rule() {
    let vertical = splitter_visual_frame(&FrameRect {
        x: 260.0,
        y: 72.0,
        width: 8.0,
        height: 378.0,
    });
    assert_eq!((vertical.x, vertical.y), (263.0, 72.0));
    assert_eq!((vertical.width, vertical.height), (1.0, 378.0));

    let horizontal = splitter_visual_frame(&FrameRect {
        x: 0.0,
        y: 450.0,
        width: 900.0,
        height: 8.0,
    });
    assert_eq!((horizontal.x, horizontal.y), (0.0, 453.0));
    assert_eq!((horizontal.width, horizontal.height), (900.0, 1.0));
}
