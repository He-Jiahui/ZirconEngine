#[path = "native_keyboard/commands.rs"]
mod commands;
#[path = "native_keyboard/dispatch.rs"]
mod dispatch;
#[path = "native_keyboard/target.rs"]
mod target;
#[cfg(test)]
#[path = "native_keyboard/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract) use commands::workbench_popup_keyboard_command;
pub(in crate::ui::retained_host::host_contract) use target::active_popup_keyboard_target_for_ui;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use commands::WorkbenchPopupKeyboardCommand;
pub(in crate::ui::retained_host::host_contract) use dispatch::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
    workbench_popup_accept_is_owned,
};
