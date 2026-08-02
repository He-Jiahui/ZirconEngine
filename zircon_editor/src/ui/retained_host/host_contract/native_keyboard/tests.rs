use super::commands::{WorkbenchPopupKeyboardCommand, workbench_popup_keyboard_command};
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
    assert_eq!(
        workbench_popup_keyboard_command(&Key::Named(NamedKey::PageDown)),
        Some(WorkbenchPopupKeyboardCommand::PageDown)
    );
    assert_eq!(
        workbench_popup_keyboard_command(&Key::Named(NamedKey::PageUp)),
        Some(WorkbenchPopupKeyboardCommand::PageUp)
    );
}
