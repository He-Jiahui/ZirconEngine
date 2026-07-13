mod discovery;
mod menu;
mod model;
mod options;
mod page_overflow;
mod search;
mod selection;

pub(in crate::ui::retained_host::host_contract) use discovery::active_popup_keyboard_target_for_ui;
pub(in crate::ui::retained_host::host_contract) use model::{
    PopupKeyboardRow, PopupKeyboardTarget,
};
pub(in crate::ui::retained_host::host_contract) use page_overflow::HOST_PAGE_OVERFLOW_DISPATCH_KIND;
