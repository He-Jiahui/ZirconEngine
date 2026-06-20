mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::prune_slots;
pub(in crate::scene::dynamic_scene::session) use preview::preview_prune_slots;
