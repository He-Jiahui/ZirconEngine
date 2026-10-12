#[path = "close_prompt_damage/entry.rs"]
mod entry;
#[path = "close_prompt_damage/union.rs"]
mod union;

pub(in crate::ui::retained_host::host_contract) use self::entry::close_prompt_action_damage_frame;
