use super::data::FrameRect;

pub(super) fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn contains_point(frame: &FrameRect, x: f32, y: f32) -> bool {
    x >= frame.x && x <= frame.x + frame.width && y >= frame.y && y <= frame.y + frame.height
}

pub(super) fn union_optional_frames(
    left: Option<FrameRect>,
    right: Option<FrameRect>,
) -> Option<FrameRect> {
    match (left, right) {
        (Some(left), Some(right)) => Some(union_frame(&left, &right)),
        (Some(frame), None) | (None, Some(frame)) => Some(frame),
        (None, None) => None,
    }
}

pub(super) fn union_frame(left: &FrameRect, right: &FrameRect) -> FrameRect {
    let x0 = left.x.min(right.x);
    let y0 = left.y.min(right.y);
    let x1 = (left.x + left.width).max(right.x + right.width);
    let y1 = (left.y + left.height).max(right.y + right.height);
    FrameRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_frame_rejects_zero_and_non_finite_frames() {
        assert!(visible_frame(&rect(0.0, 0.0, 1.0, 1.0)));
        assert!(!visible_frame(&rect(0.0, 0.0, 0.0, 1.0)));
        assert!(!visible_frame(&rect(f32::NAN, 0.0, 1.0, 1.0)));
    }

    #[test]
    fn union_optional_frames_preserves_single_frame_and_unions_pairs() {
        let left = rect(4.0, 8.0, 12.0, 10.0);
        let right = rect(10.0, 2.0, 30.0, 14.0);

        assert_eq!(union_optional_frames(Some(left.clone()), None), Some(left));
        assert_eq!(
            union_optional_frames(Some(rect(4.0, 8.0, 12.0, 10.0)), Some(right)),
            Some(rect(4.0, 2.0, 36.0, 16.0))
        );
        assert_eq!(union_optional_frames(None, None), None);
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
