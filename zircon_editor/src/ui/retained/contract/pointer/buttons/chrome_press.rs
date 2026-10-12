#[path = "chrome_press/entry.rs"]
mod entry;
#[path = "chrome_press/floating.rs"]
mod floating;
#[path = "chrome_press/rails.rs"]
mod rails;
#[path = "chrome_press/resize.rs"]
mod resize;
#[path = "chrome_press/tabs.rs"]
mod tabs;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_chrome_press;
