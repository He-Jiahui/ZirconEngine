mod entry;
mod floating;
mod rails;
mod resize;
mod tabs;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_chrome_press;
