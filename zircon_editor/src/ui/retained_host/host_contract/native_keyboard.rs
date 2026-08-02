mod commands;
mod dispatch;
mod target;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use commands::WorkbenchPopupKeyboardCommand;
pub(in crate::ui::retained_host::host_contract) use commands::workbench_popup_keyboard_command;
pub(in crate::ui::retained_host::host_contract) use dispatch::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
};
