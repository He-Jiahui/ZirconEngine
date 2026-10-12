#[path = "asset/content.rs"]
mod content;
#[path = "asset/entry.rs"]
mod entry;
#[path = "asset/reference.rs"]
mod reference;
#[path = "asset/tree.rs"]
mod tree;

pub(super) use self::entry::dispatch_asset_pane_target_button;
