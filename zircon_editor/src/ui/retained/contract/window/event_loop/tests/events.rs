use winit::event::{ButtonSource, ElementState, MouseButton, WindowEvent};

use super::mouse_button_pressed;

#[test]
fn routed_native_inputs_keep_translation_identity_until_their_handler() {
    let source = include_str!("../events.rs");

    assert!(source.contains("then(|| self.translate_platform_input_event(&event))"));
    assert!(!source.contains(".flatten()"));
    assert!(source.contains("require_platform_input(platform_input_event)"));
    assert!(source.contains("self.begin_input_outcome(platform_event.sequence)"));
    assert!(source.contains("self.reject_input_outcome()"));
}

#[test]
fn native_focus_loss_routes_to_the_viewport_interaction_cancellation_callback() {
    let source = include_str!("../events.rs");

    assert!(source.contains("WindowEvent::Focused(false)"));
    assert!(source.contains("handle_native_window_focus_lost"));
}

#[test]
fn native_focus_gain_routes_to_the_owner_acknowledgement_callback() {
    let source = include_str!("../events.rs");

    assert!(source.contains("WindowEvent::Focused(true)"));
    assert!(source.contains("handle_native_window_focused"));
}

#[test]
fn only_mouse_buttons_gate_idle_move_coalescing() {
    let mouse = WindowEvent::PointerButton {
        device_id: None,
        state: ElementState::Pressed,
        position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        primary: true,
        button: ButtonSource::Mouse(MouseButton::Left),
    };
    let unknown = WindowEvent::PointerButton {
        device_id: None,
        state: ElementState::Pressed,
        position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        primary: true,
        button: ButtonSource::Unknown(1),
    };

    assert_eq!(mouse_button_pressed(&mouse), Some(true));
    assert_eq!(mouse_button_pressed(&unknown), None);
}
