#[path = "popup/entry.rs"]
mod entry;
#[path = "popup/nested.rs"]
mod nested;
#[path = "../popup/submenu_frame.rs"]
mod submenu_frame;

pub(in crate::ui::retained_host::host_contract) use self::entry::menu_popup_handles_point_with_state;
