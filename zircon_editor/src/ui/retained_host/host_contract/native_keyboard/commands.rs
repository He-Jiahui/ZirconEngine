use winit::keyboard::{Key, NamedKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum WorkbenchPopupKeyboardCommand {
    Next,
    Previous,
    First,
    Last,
    PageDown,
    PageUp,
    Accept,
    Cancel,
}

pub(in crate::ui::retained_host::host_contract) fn workbench_popup_keyboard_command(
    key: &Key,
) -> Option<WorkbenchPopupKeyboardCommand> {
    match key {
        Key::Named(NamedKey::ArrowDown) => Some(WorkbenchPopupKeyboardCommand::Next),
        Key::Named(NamedKey::ArrowUp) => Some(WorkbenchPopupKeyboardCommand::Previous),
        Key::Named(NamedKey::Home) => Some(WorkbenchPopupKeyboardCommand::First),
        Key::Named(NamedKey::End) => Some(WorkbenchPopupKeyboardCommand::Last),
        Key::Named(NamedKey::PageDown) => Some(WorkbenchPopupKeyboardCommand::PageDown),
        Key::Named(NamedKey::PageUp) => Some(WorkbenchPopupKeyboardCommand::PageUp),
        Key::Named(NamedKey::Enter) => Some(WorkbenchPopupKeyboardCommand::Accept),
        Key::Named(NamedKey::Escape) => Some(WorkbenchPopupKeyboardCommand::Cancel),
        _ => None,
    }
}
