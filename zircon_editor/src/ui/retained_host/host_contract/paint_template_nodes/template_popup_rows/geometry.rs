use super::super::super::data::FrameRect;

pub(super) fn has_paintable_popup_row_extent(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    has_paintable_popup_row_extent(outer)
        && has_paintable_popup_row_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_row_frame_rejects_collapsed_non_finite_and_outside_geometry() {
        let outer = FrameRect {
            x: 4.0,
            y: 8.0,
            width: 24.0,
            height: 18.0,
        };

        assert!(frame_is_within(
            &outer,
            &FrameRect {
                x: 5.0,
                y: 9.0,
                width: 12.0,
                height: 8.0,
            }
        ));
        assert!(!has_paintable_popup_row_extent(&FrameRect {
            width: 0.0,
            ..outer.clone()
        }));
        assert!(!has_paintable_popup_row_extent(&FrameRect {
            x: f32::NAN,
            ..outer.clone()
        }));
        assert!(!frame_is_within(
            &outer,
            &FrameRect {
                x: 20.0,
                y: 9.0,
                width: 12.0,
                height: 8.0,
            }
        ));
    }
}
