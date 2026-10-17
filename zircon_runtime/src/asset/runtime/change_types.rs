#[path = "../watch/asset_change.rs"]
mod asset_change;
#[path = "../watch/asset_change_construction.rs"]
mod asset_change_construction;
#[path = "../watch/asset_change_kind.rs"]
mod asset_change_kind;
pub use asset_change::AssetChange;
pub use asset_change_kind::AssetChangeKind;
