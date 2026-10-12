#[path = "panes/entry.rs"]
mod entry;
#[path = "panes/mode.rs"]
mod mode;
#[path = "panes/pane.rs"]
mod pane;
#[path = "panes/target.rs"]
mod target;

pub(in crate::ui::retained_host::host_contract) use self::entry::{
    route_pointer_move_to_pane, route_pointer_scroll_to_pane, route_pointer_to_pane,
};
