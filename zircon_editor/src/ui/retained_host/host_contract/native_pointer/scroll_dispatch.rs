mod dock_overflow;
mod entry;
mod menu;
mod page_overflow;
mod pane;
mod settings;
mod viewport;

pub(in crate::ui::retained_host::host_contract) use self::entry::dispatch_native_pointer_scroll;
