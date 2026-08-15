use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn normalized_presentation_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

pub(in super::super) fn logical_axis_from_physical(
    physical_value: f32,
    physical_origin: f32,
    scale_factor: f32,
) -> f32 {
    (physical_value - physical_origin) / normalized_presentation_scale_factor(scale_factor)
}

pub(super) fn scale_frame(frame: UiFrame, scale_factor: f32) -> UiFrame {
    let scale_factor = normalized_presentation_scale_factor(scale_factor);
    UiFrame::new(
        frame.x * scale_factor,
        frame.y * scale_factor,
        frame.width * scale_factor,
        frame.height * scale_factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_pointer_coordinates_cross_the_inverse_scale_boundary_once() {
        assert_eq!(logical_axis_from_physical(148.0, 100.0, 2.0), 24.0);
        assert_eq!(logical_axis_from_physical(148.0, 100.0, 0.0), 48.0);
    }

    #[test]
    fn logical_frames_cross_the_physical_boundary_once() {
        assert_eq!(
            scale_frame(UiFrame::new(8.0, 12.0, 80.0, 24.0), 2.0),
            UiFrame::new(16.0, 24.0, 160.0, 48.0)
        );
    }
}
