use super::super::data::FrameRect;
use super::frame::is_visible_frame;

pub(in crate::ui::retained_host::host_contract) fn translated(
    frame: &FrameRect,
    origin_x: f32,
    origin_y: f32,
) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

pub(in crate::ui::retained_host::host_contract) fn intersect(
    left: &FrameRect,
    right: &FrameRect,
) -> Option<FrameRect> {
    if !is_visible_frame(left) || !is_visible_frame(right) {
        return None;
    }

    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    let width = x1 - x0;
    let height = y1 - y0;
    (width > 0.5 && height > 0.5).then_some(FrameRect {
        x: x0,
        y: y0,
        width,
        height,
    })
}

pub(in crate::ui::retained_host::host_contract) fn inset(
    frame: &FrameRect,
    amount: f32,
) -> FrameRect {
    let amount = amount.max(0.0);
    FrameRect {
        x: frame.x + amount,
        y: frame.y + amount,
        width: (frame.width - amount * 2.0).max(0.0),
        height: (frame.height - amount * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract) fn corner_radius_for_frame(
    frame: &FrameRect,
    requested_radius: f32,
) -> f32 {
    if !frame.width.is_finite() || !frame.height.is_finite() || !requested_radius.is_finite() {
        return 0.0;
    }
    requested_radius
        .max(0.0)
        .min(bounded_extent(frame.width).min(bounded_extent(frame.height)) * 0.5)
}

pub(in crate::ui::retained_host::host_contract) fn bounded_extent(value: f32) -> f32 {
    value.is_finite().then_some(value.max(0.0)).unwrap_or(0.0)
}

// Pixel snapping may only remove coverage; expanding a child would bleed across adjacent panels.
pub(in crate::ui::retained_host::host_contract) fn inward_pixel_aligned_rect(
    frame: &FrameRect,
) -> FrameRect {
    let right = frame.x + frame.width;
    let bottom = frame.y + frame.height;
    if !frame.x.is_finite() || !frame.y.is_finite() || !right.is_finite() || !bottom.is_finite() {
        return FrameRect {
            x: frame.x,
            y: frame.y,
            width: 0.0,
            height: 0.0,
        };
    }
    let x = frame.x.ceil();
    let y = frame.y.ceil();
    FrameRect {
        x,
        y,
        width: (right.floor() - x).max(0.0),
        height: (bottom.floor() - y).max(0.0),
    }
}

#[cfg(test)]
mod tests {
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
}
