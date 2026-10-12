#[path = "geometry/popup.rs"]
mod popup;
#[path = "geometry/row.rs"]
mod row;
#[path = "geometry/scroll.rs"]
mod scroll;
#[path = "geometry/shell.rs"]
mod shell;
#[path = "geometry/submenu.rs"]
mod submenu;

pub(in crate::ui::retained_host::host_contract) use popup::{
    constrained_menu_popup_frame, menu_popup_height,
};
pub(in crate::ui::retained_host::host_contract) use row::menu_popup_row_frame;
pub(in crate::ui::retained_host::host_contract) use scroll::scrolled_menu_frame;
pub(in crate::ui::retained_host::host_contract) use submenu::constrained_submenu_popup_frame;
