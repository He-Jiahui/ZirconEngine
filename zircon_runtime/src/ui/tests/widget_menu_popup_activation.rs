use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchDisposition, UiInputEvent},
    event_ui::UiNodeId,
};

use super::widget_menu_behavior::{
    assert_popup_open, assert_popup_stack, keyboard_pressed, menu_surface_closed,
};

#[test]
fn popup_keyboard_activation_updates_shared_popup_stack() {
    let mut surface = menu_surface_closed();
    assert_popup_stack(&surface, &[]);
    assert!(!surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .supports_pointer());
    surface.focus.focused = Some(UiNodeId::new(2));

    let opened = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Enter", 13)),
        )
        .unwrap();

    assert_eq!(opened.reply.disposition, UiDispatchDisposition::Handled);
    assert_popup_open(&surface, true);
    assert_popup_stack(&surface, &["root/popup"]);

    let closed = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(keyboard_pressed("Space", 32)),
        )
        .unwrap();

    assert_eq!(closed.reply.disposition, UiDispatchDisposition::Handled);
    assert_popup_open(&surface, false);
    assert_popup_stack(&surface, &[]);
}
