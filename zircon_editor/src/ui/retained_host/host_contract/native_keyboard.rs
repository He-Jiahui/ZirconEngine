mod commands;
mod dispatch;
mod target;
#[cfg(test)]
mod tests;

pub(super) use commands::workbench_popup_keyboard_command;
#[cfg(test)]
pub(super) use commands::WorkbenchPopupKeyboardCommand;
pub(super) use dispatch::{
    dispatch_workbench_popup_keyboard_command, dispatch_workbench_popup_text_search,
};
