use super::commands::{workbench_popup_keyboard_command, WorkbenchPopupKeyboardCommand};
use winit::keyboard::{Key, NamedKey};

#[test]
fn workbench_popup_keyboard_command_maps_boundary_keys() {
    assert_eq!(
        workbench_popup_keyboard_command(&Key::Named(NamedKey::Home)),
        Some(WorkbenchPopupKeyboardCommand::First)
    );
    assert_eq!(
        workbench_popup_keyboard_command(&Key::Named(NamedKey::End)),
        Some(WorkbenchPopupKeyboardCommand::Last)
    );
}
