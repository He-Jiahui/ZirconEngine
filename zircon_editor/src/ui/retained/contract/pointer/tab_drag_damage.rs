#[path = "tab_drag_damage/entry.rs"]
mod entry;
#[path = "tab_drag_damage/floating.rs"]
mod floating;
#[path = "tab_drag_damage/groups.rs"]
mod groups;
#[path = "tab_drag_damage/regions.rs"]
mod regions;
#[path = "tab_drag_damage/union.rs"]
mod union;

pub(in crate::ui::retained_host::host_contract) use self::entry::tab_drag_release_damage_frame;
