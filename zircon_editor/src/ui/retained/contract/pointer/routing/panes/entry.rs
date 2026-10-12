#[path = "entry/floating.rs"]
mod floating;
#[path = "entry/local.rs"]
mod local;
#[path = "entry/route.rs"]
mod route;

pub(in crate::ui::retained_host::host_contract) use self::route::{
    route_pointer_move_to_pane, route_pointer_scroll_to_pane, route_pointer_to_pane,
};
