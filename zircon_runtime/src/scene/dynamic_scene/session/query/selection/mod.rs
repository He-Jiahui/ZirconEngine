mod global;
mod tag;
mod typed;

pub(in crate::scene::dynamic_scene::session) use global::{
    latest_updated_slot_id, oldest_updated_slot_id,
};
pub(in crate::scene::dynamic_scene::session) use tag::{
    latest_updated_slot_id_with_tag, oldest_updated_slot_id_with_tag,
};
pub(in crate::scene::dynamic_scene::session) use typed::select_slot;
