#[path = "scroll/dock_overflow.rs"]
mod dock_overflow;
#[path = "scroll/entry.rs"]
mod entry;
#[path = "scroll/menu.rs"]
mod menu;
#[path = "scroll/page_overflow.rs"]
mod page_overflow;
#[path = "scroll/pane.rs"]
mod pane;
#[path = "scroll/settings.rs"]
mod settings;
#[path = "scroll/viewport.rs"]
mod viewport;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_native_pointer_scroll;
