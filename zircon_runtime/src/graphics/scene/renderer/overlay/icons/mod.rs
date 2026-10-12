#[path = "icon_entry.rs"]
mod icon_entry;
#[path = "icon_slot.rs"]
mod icon_slot;
#[path = "../../../overlay/icons/viewport_icon_atlas/mod.rs"]
mod viewport_icon_atlas;
#[path = "viewport_icon_sprite.rs"]
mod viewport_icon_sprite;

pub(crate) use viewport_icon_atlas::ViewportIconAtlas;
