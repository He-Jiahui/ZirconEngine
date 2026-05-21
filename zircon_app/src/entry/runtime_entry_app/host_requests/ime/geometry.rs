use winit::dpi::{LogicalPosition, LogicalSize};
use zircon_runtime_interface::ZrRuntimeImeCursorAreaV1;

pub(super) fn ime_logical_position(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Position {
    LogicalPosition::new(area.x as f64, area.y as f64).into()
}

pub(super) fn ime_logical_size(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Size {
    LogicalSize::new(area.width as f64, area.height as f64).into()
}
