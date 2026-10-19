use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiPointerEvent,
    UiPointerInputEvent,
};
use zircon_runtime_interface::ui::layout::UiPoint;
use zircon_runtime_interface::ui::window::UiWindowInputPumpEvent;

fn event(sequence: u64, kind: UiPointerEventKind, x: f32, y: f32) -> PlatformInputTranslation {
    let mut event = UiPointerEvent::new(kind, UiPoint::new(x, y));
    event.button = Some(UiPointerButton::Primary);
    event.scroll_delta = 1.0;
    PlatformInputTranslation {
        sequence: UiInputSequence::new(sequence),
        event: Some(UiWindowInputPumpEvent::Input(UiInputEvent::Pointer(UiPointerInputEvent {
            metadata: UiInputEventMetadata::new(UiInputTimestamp::from_micros(0), UiInputSequence::new(sequence)),
            event,
            precise_scroll: None,
        }))),
    }
}

#[test]
fn a_real_click_at_the_window_origin_replaces_the_old_position() {
    let host = UiHostWindow::new().unwrap();
    let points = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&points);
    host.global::<UiHostContext>().on_workbench_pointer_input(move |pointer, _| {
        captured.borrow_mut().push(pointer.event.point);
    });
    let mut window = UiHostWindowEventLoop::new(host);
    window.last_pointer_position = Some((400.0, 300.0));
    window.handle_pointer_button(event(1, UiPointerEventKind::Down, 0.0, 0.0));
    assert_eq!(window.last_pointer_position, Some((0.0, 0.0)));
    assert_eq!(points.borrow().as_slice(), &[UiPoint::new(0.0, 0.0)]);
}

#[test]
fn a_wheel_without_a_known_position_is_not_routed_to_a_control() {
    let host = UiHostWindow::new().unwrap();
    let points = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&points);
    host.global::<UiHostContext>().on_workbench_pointer_input(move |pointer, _| {
        captured.borrow_mut().push(pointer.event.point);
    });
    let mut window = UiHostWindowEventLoop::new(host);
    window.handle_mouse_wheel(event(1, UiPointerEventKind::Scroll, 0.0, 0.0), MouseScrollDelta::LineDelta(0.0, 1.0));
    assert!(points.borrow().is_empty());
    window.last_pointer_position = Some((24.0, 48.0));
    window.handle_mouse_wheel(event(2, UiPointerEventKind::Scroll, 0.0, 0.0), MouseScrollDelta::LineDelta(0.0, 1.0));
    assert_eq!(points.borrow().as_slice(), &[UiPoint::new(24.0, 48.0)]);
    window.handle_native_window_focus_lost();
    assert_eq!(window.last_pointer_position, None);
    window.handle_mouse_wheel(event(3, UiPointerEventKind::Scroll, 0.0, 0.0), MouseScrollDelta::LineDelta(0.0, 1.0));
    assert_eq!(points.borrow().len(), 1);
}
