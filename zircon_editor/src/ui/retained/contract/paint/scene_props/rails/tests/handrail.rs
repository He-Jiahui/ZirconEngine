use super::*;

#[test]
fn handrail_posts_follow_the_rail_width_and_available_clip_height() {
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 100.0,
    };
    let standard = FrameRect {
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 4.0,
    };
    let wide = FrameRect {
        width: 200.0,
        ..standard.clone()
    };
    let narrow = FrameRect {
        width: 24.0,
        ..standard.clone()
    };
    let subpixel = FrameRect {
        width: 0.5,
        ..standard.clone()
    };
    let short_clip = FrameRect {
        height: 30.0,
        ..clip.clone()
    };

    let standard_posts = handrail_post_rects(&standard, &clip);
    let wide_posts = handrail_post_rects(&wide, &clip);
    let narrow_posts = handrail_post_rects(&narrow, &clip);
    let subpixel_posts = handrail_post_rects(&subpixel, &clip);
    let clipped_posts = handrail_post_rects(&standard, &short_clip);

    assert_eq!(standard_posts[0].x, 46.0);
    assert_eq!(standard_posts[1].x, 68.0);
    assert_eq!(standard_posts[0].height, 56.0);
    assert_eq!(wide_posts[0].x, 82.0);
    assert_eq!(wide_posts[1].x, 126.0);
    assert!(narrow_posts[0].x >= narrow.x);
    assert!(narrow_posts[1].right() <= narrow.right());
    assert!(subpixel_posts[0].right() <= subpixel.right());
    assert!(subpixel_posts[1].right() <= subpixel.right());
    assert_eq!(clipped_posts[0].bottom(), short_clip.bottom());
}
