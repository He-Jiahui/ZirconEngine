mod discovery;
mod menu;
mod model;
mod options;
mod search;
mod selection;

pub(in crate::ui::retained_host::host_contract) use discovery::active_popup_keyboard_target_for_ui;
pub(in crate::ui::retained_host::host_contract) use model::{
    PopupKeyboardRow, PopupKeyboardTarget,
};
