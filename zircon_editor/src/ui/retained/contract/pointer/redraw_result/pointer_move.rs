#[path = "../pointer_move/entry.rs"]
mod entry;
#[path = "../pointer_move/hierarchy.rs"]
mod hierarchy;
#[path = "../pointer_move/template.rs"]
mod template;

pub(in crate::ui::retained_host::host_contract) use self::entry::pointer_move_redraw;
