#[path = "target/discovery.rs"]
mod discovery;
#[path = "target/dock_overflow.rs"]
mod dock_overflow;
#[path = "target/menu.rs"]
mod menu;
#[path = "target/model.rs"]
mod model;
#[path = "target/options.rs"]
mod options;
#[path = "target/page_overflow.rs"]
mod page_overflow;
#[path = "target/search.rs"]
mod search;
#[path = "target/selection.rs"]
mod selection;

pub(in crate::ui::retained_host::host_contract) use discovery::active_popup_keyboard_target_for_ui;
pub(in crate::ui::retained_host::host_contract) use dock_overflow::HOST_DOCK_OVERFLOW_DISPATCH_KIND;
pub(in crate::ui::retained_host::host_contract) use model::{
    PopupKeyboardMove, PopupKeyboardRow, PopupKeyboardTarget, PopupKeyboardWindowFocus,
    PopupKeyboardWindowRequest,
};
pub(in crate::ui::retained_host::host_contract) use page_overflow::HOST_PAGE_OVERFLOW_DISPATCH_KIND;
