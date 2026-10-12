#[path = "pane/asset.rs"]
mod asset;
#[path = "pane/entry.rs"]
mod entry;
#[path = "pane/native.rs"]
mod native;
#[path = "pane/passive.rs"]
mod passive;
#[path = "pane/template.rs"]
mod template;
#[path = "pane/viewport.rs"]
mod viewport;

pub(super) use self::entry::dispatch_pane_pointer_move;
