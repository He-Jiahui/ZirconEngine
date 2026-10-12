#[path = "../../../../../contract/pointer/routing/chrome/top_level/docked.rs"]
mod docked;
#[path = "../../../../../contract/pointer/routing/chrome/top_level/drawers.rs"]
mod drawers;
#[path = "top_level/entry.rs"]
mod entry;
#[path = "../../../../../contract/pointer/routing/chrome/top_level/resize.rs"]
mod resize;
#[path = "../../../../../contract/pointer/routing/chrome/top_level/side_rails.rs"]
mod side_rails;

pub(in crate::ui::retained_host::host_contract) use self::entry::route_top_level_chrome;
