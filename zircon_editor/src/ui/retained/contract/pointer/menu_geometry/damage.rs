#[path = "damage/entry.rs"]
mod entry;
#[path = "../damage/popup_bottom.rs"]
mod popup_bottom;
#[path = "damage/stack.rs"]
mod stack;

pub(in crate::ui::retained_host::host_contract) use self::entry::menu_damage_frame_with_state;
