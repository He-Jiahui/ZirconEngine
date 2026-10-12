#[path = "chrome/floating.rs"]
mod floating;
#[path = "chrome/rails.rs"]
mod rails;
#[path = "chrome/tabs.rs"]
mod tabs;
#[path = "chrome/top_level.rs"]
mod top_level;

pub(in crate::ui::retained_host::host_contract) use self::top_level::route_top_level_chrome;
