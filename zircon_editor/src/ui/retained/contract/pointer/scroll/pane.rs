#[path = "pane/asset.rs"]
mod asset;
#[path = "pane/entry.rs"]
mod entry;
#[path = "pane/native.rs"]
mod native;
#[path = "pane/passive.rs"]
mod passive;
#[path = "pane/viewport_target.rs"]
mod viewport_target;

pub(super) use self::entry::dispatch_pane_pointer_scroll;
