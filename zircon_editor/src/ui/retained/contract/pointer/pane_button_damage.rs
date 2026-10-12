#[path = "pane_button_damage/entry.rs"]
mod entry;
#[path = "pane_button_damage/union.rs"]
mod union;

pub(in crate::ui::retained_host::host_contract) use self::entry::pane_pointer_press_damage_frame;
