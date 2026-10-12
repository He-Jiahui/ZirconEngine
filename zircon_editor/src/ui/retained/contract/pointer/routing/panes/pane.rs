#[path = "pane/entry.rs"]
mod entry;
#[path = "pane/toolbar.rs"]
mod toolbar;

pub(super) use self::entry::pane_route_from_pane;
