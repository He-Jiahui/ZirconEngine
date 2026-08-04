use winit::dpi::{LogicalPosition, LogicalSize};
use zircon_runtime_interface::ZrRuntimeImeCursorAreaV1;

pub(super) fn ime_logical_cursor_area(
    area: ZrRuntimeImeCursorAreaV1,
) -> Option<(winit::dpi::Position, winit::dpi::Size)> {
    (area.x.is_finite()
        && area.y.is_finite()
        && area.width.is_finite()
        && area.height.is_finite()
        && area.width >= 0.0
        && area.height >= 0.0)
        .then(|| (ime_logical_position(area), ime_logical_size(area)))
}

pub(super) fn ime_logical_position(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Position {
    LogicalPosition::new(area.x as f64, area.y as f64).into()
}

pub(super) fn ime_logical_size(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Size {
    LogicalSize::new(area.width as f64, area.height as f64).into()
}

#[cfg(test)]
mod tests {
    use winit::dpi::{LogicalPosition, LogicalSize, Position, Size};
    use zircon_runtime_interface::ZrRuntimeImeCursorAreaV1;

    use super::{ime_logical_cursor_area, ime_logical_position, ime_logical_size};

    #[test]
    fn ime_cursor_area_submits_window_logical_coordinates_without_dpi_scaling() {
        let area = ZrRuntimeImeCursorAreaV1::new(25.0, 68.5, 4.0, 37.0);

        match ime_logical_position(area) {
            Position::Logical(position) => {
                assert_eq!(position, LogicalPosition::new(25.0, 68.5));
            }
            position => panic!("expected logical IME position, got {position:?}"),
        }
        match ime_logical_size(area) {
            Size::Logical(size) => {
                assert_eq!(size, LogicalSize::new(4.0, 37.0));
            }
            size => panic!("expected logical IME size, got {size:?}"),
        }
    }

    #[test]
    fn ime_cursor_area_rejects_non_finite_or_negative_size_geometry() {
        for area in [
            ZrRuntimeImeCursorAreaV1::new(f32::NAN, 68.5, 4.0, 37.0),
            ZrRuntimeImeCursorAreaV1::new(25.0, f32::INFINITY, 4.0, 37.0),
            ZrRuntimeImeCursorAreaV1::new(25.0, 68.5, -4.0, 37.0),
            ZrRuntimeImeCursorAreaV1::new(25.0, 68.5, 4.0, f32::NEG_INFINITY),
        ] {
            assert!(
                ime_logical_cursor_area(area).is_none(),
                "invalid ABI cursor geometry must not reach the window API: {area:?}"
            );
        }
    }
}
