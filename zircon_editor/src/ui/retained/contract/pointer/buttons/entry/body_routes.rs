#[path = "../../../../../contract/pointer/buttons/entry/body_routes/entry.rs"]
mod entry;
#[path = "../../../../../contract/pointer/buttons/entry/body_routes/fallback.rs"]
mod fallback;
#[path = "../../../../../contract/pointer/buttons/entry/body_routes/input.rs"]
mod input;
#[path = "body_routes/pane.rs"]
mod pane;
#[path = "../../../../../contract/pointer/buttons/entry/body_routes/sequence.rs"]
mod sequence;
#[path = "../../../../../contract/pointer/buttons/entry/body_routes/workbench.rs"]
mod workbench;

pub(super) use self::entry::dispatch_body_button_routes;
