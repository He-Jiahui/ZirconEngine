use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiPointerEvent, UiPointerInputEvent,
        UiPreciseScrollDelta,
    },
    layout::UiPoint,
    surface::UiPointerEventKind,
};

const PIXEL_SCROLL_LEGACY_LINE_SCALE: f32 = 0.1;

pub(crate) fn native_mouse_wheel_event_to_shared_input(
    metadata: UiInputEventMetadata,
    point: UiPoint,
    delta: MouseScrollDelta,
) -> UiInputEvent {
    let (scroll_delta, precise_scroll) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (y, UiPreciseScrollDelta::lines(x, y)),
        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
            let x = x as f32;
            let y = y as f32;
            (
                y * PIXEL_SCROLL_LEGACY_LINE_SCALE,
                UiPreciseScrollDelta::pixels(x, y),
            )
        }
    };

    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(UiPointerEventKind::Scroll, point)
            .with_scroll_delta(scroll_delta),
        precise_scroll: Some(precise_scroll),
    })
}
