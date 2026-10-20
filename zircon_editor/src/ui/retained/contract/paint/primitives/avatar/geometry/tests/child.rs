use super::avatar_fallback_child_frame;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn avatar_fallback_child_stays_within_subpixel_and_empty_parents() {
    for (width, height) in [
        (0.0, 0.0),
        (0.0, 2.0),
        (0.25, 0.75),
        (0.5, 0.125),
        (0.75, 0.25),
        (0.999, 0.2),
    ] {
        let parent = FrameRect {
            x: 10.0,
            y: -5.0,
            width,
            height,
        };
        let child = avatar_fallback_child_frame(&parent);
        assert!(child.width >= 0.0 && child.width <= width);
        assert!(child.height >= 0.0 && child.height <= height);
        assert!(child.x >= parent.x - 0.00001);
        assert!(child.y >= parent.y - 0.00001);
        assert!(child.x + child.width <= parent.x + width + 0.00001);
        assert!(child.y + child.height <= parent.y + height + 0.00001);
        if width == 0.0 || height == 0.0 {
            assert_eq!((child.width, child.height), (0.0, 0.0));
        }
    }
}

#[test]
fn avatar_fallback_child_preserves_centering_and_existing_minimum_when_it_fits() {
    for (width, height) in [(1.0, 4.0), (32.0, 64.0), (64.0, 32.0)] {
        let parent = FrameRect {
            x: -12.0,
            y: 18.0,
            width,
            height,
        };
        let child = avatar_fallback_child_frame(&parent);
        assert!(child.width >= 1.0 && child.width <= width.min(height));
        assert_eq!(child.width, child.height);
        assert!((child.x + child.width * 0.5 - (parent.x + width * 0.5)).abs() < 0.00001);
        assert!((child.y + child.height * 0.5 - (parent.y + height * 0.5)).abs() < 0.00001);
        if width == 1.0 {
            assert_eq!((child.width, child.height), (1.0, 1.0));
        }
    }
}
