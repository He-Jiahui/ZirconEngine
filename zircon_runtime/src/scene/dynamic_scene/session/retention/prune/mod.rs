mod global;
mod planning;
mod tag;

pub(in crate::scene::dynamic_scene::session) use global::{preview_prune_slots, prune_slots};
pub(in crate::scene::dynamic_scene::session) use tag::{
    preview_prune_slots_with_tag, prune_slots_with_tag,
};
