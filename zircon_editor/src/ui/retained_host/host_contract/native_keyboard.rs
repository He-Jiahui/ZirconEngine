mod commands;
mod dispatch;
mod target;
#[cfg(test)]
#[path = "native_keyboard/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract) use commands::workbench_popup_keyboard_command;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use commands::WorkbenchPopupKeyboardCommand;
pub(in crate::ui::retained_host::host_contract) use dispatch::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
    workbench_popup_accept_is_owned,
};
